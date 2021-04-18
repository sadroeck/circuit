use crate::proc::Proc;
use crate::proc_ext::ProcExt;
use crate::{thread, tokio};
use flume::{bounded, Receiver, Sender};

pub fn with_worker_pool<I, O, F>(
    workers: usize,
    channel_capacity: usize,
    in_r: Receiver<I>,
    out_s: Sender<O>,
    work_fn: F,
) -> impl Proc
where
    I: Send + 'static,
    O: Send + 'static,
    F: Fn(usize, Receiver<(I, Sender<O>)>) + Copy + Send + 'static,
{
    assert!(workers >= 1);
    let (work_dispatch_s, work_dispatch_r) = bounded(channel_capacity);
    let (work_collect_s, work_collect_r) = bounded(channel_capacity);
    let dispatch = tokio(async move {
        // dispatch work to workers
        let dispatch = tokio::spawn(async move {
            while let Ok(msg) = in_r.recv_async().await {
                let (s, r) = flume::bounded(1);
                if work_collect_s.send_async(r).await.is_err() {
                    break;
                }
                if work_dispatch_s.send_async((msg, s)).await.is_err() {
                    break;
                }
            }
        });
        // collect output from workers
        let collect = tokio::spawn(async move {
            while let Ok(r) = work_collect_r.recv_async().await {
                if let Ok(output) = r.recv_async().await {
                    if out_s.send_async(output).await.is_err() {
                        break;
                    }
                }
            }
        });
        dispatch.await?;
        collect.await?;
        Ok(())
    });

    (0..workers)
        .map(|worker_id| {
            let work_r = work_dispatch_r.clone();
            thread(move || {
                work_fn(worker_id, work_r);
                Ok(())
            })
        })
        .fold(dispatch.boxed(), |x, y| x.and(y).boxed())
}
