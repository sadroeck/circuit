use crate::proc::Proc;
use crate::runners::runtime::TaskRuntime;
use futures::future::JoinAll;
use std::future::IntoFuture;
use tokio::runtime::Handle;
use tokio::task::{JoinError, JoinHandle};

pub struct JoinTasks<T: Send + 'static> {
    runtime: TaskRuntime,
    tasks: Vec<JoinHandle<T>>,
}

impl<T: Send + 'static> Default for JoinTasks<T> {
    #[inline]
    fn default() -> Self {
        let runtime = Handle::try_current()
            .map(TaskRuntime::Entered)
            .unwrap_or_else(|_| TaskRuntime::new());
        Self {
            runtime,
            tasks: Default::default(),
        }
    }
}

impl<T: Send + 'static> JoinTasks<T> {
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn with_runtime(handle: Handle) -> Self {
        Self {
            runtime: TaskRuntime::Entered(handle),
            tasks: Default::default(),
        }
    }

    #[inline]
    pub fn and<F>(mut self, fut: F) -> Self
    where
        F: IntoFuture<Output = T>,
        <F as IntoFuture>::IntoFuture: Send + 'static,
    {
        self.tasks
            .push(self.runtime.handle().spawn(fut.into_future()));
        self
    }
}

impl<T: Send + 'static> IntoFuture for JoinTasks<T> {
    type Output = Vec<Result<T, JoinError>>;
    type IntoFuture = JoinAll<JoinHandle<T>>;

    fn into_future(mut self) -> Self::IntoFuture {
        let tasks = std::mem::take(&mut self.tasks);
        futures::future::join_all(tasks)
    }
}

impl<T: Send + 'static> Proc for JoinTasks<T> {
    type Output = Vec<T>;

    fn join(&mut self) -> anyhow::Result<Self::Output> {
        if self.tasks.is_empty() {
            return Ok(Vec::new());
        }
        let tasks = std::mem::take(&mut self.tasks);
        let (output_tx, output_rx) = flume::bounded(1);
        self.runtime.handle().spawn(async move {
            let res = futures::future::join_all(tasks)
                .await
                .into_iter()
                .map(|res| res.map_err(anyhow::Error::from))
                .collect::<anyhow::Result<Vec<_>>>();
            let _ = output_tx.send_async(res).await;
        });
        output_rx.recv()?
    }

    #[inline]
    fn forget(&mut self) {
        for task in self.tasks.drain(..) {
            task.abort();
        }
    }
}

impl<T: Send + 'static> Drop for JoinTasks<T> {
    fn drop(&mut self) {
        let _ = self.join();
        self.runtime.shutdown();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tokio;
    use std::time::Duration;

    #[test]
    fn join_2() {
        let results = JoinTasks::new()
            .and(async move { 1 })
            .and(async move { 2 })
            .join()
            .expect("could not join");
        assert_eq!(results, vec![1, 2])
    }

    #[test]
    fn join_3() {
        let results = JoinTasks::new()
            .and(async move { 1 })
            .and(async move { 2 })
            .and(async move { 3 })
            .join()
            .expect("could not join");
        assert_eq!(results, vec![1, 2, 3])
    }

    #[test]
    fn join_0() {
        let results = JoinTasks::<()>::new().join().expect("could not join");
        assert_eq!(results, vec![])
    }

    #[tokio::test]
    async fn into_future() {
        let tasks = JoinTasks::new().and(async move { 1 }).and(async move { 2 });
        let results = tasks.into_future().await;
        let results = results
            .into_iter()
            .filter_map(Result::ok)
            .collect::<Vec<_>>();
        assert_eq!(results, vec![1, 2])
    }

    #[test]
    fn inside_proc() {
        tokio(async {
            tokio::task::spawn_blocking(|| {
                let results = JoinTasks::new()
                    .and(async move { 1 })
                    .and(async move { 2 })
                    .join()
                    .expect("could not join");
                assert_eq!(results, vec![1, 2]);
            })
            .await
            .map_err(anyhow::Error::from)
        })
        .join()
        .expect("Could not join")
    }

    /// Note: This is generally a terrible idea as this will block one of the executor threads
    /// But it's functional..
    /// If spawned inside a single-threaded runtime, it will block the runtime
    #[tokio::test(flavor = "multi_thread")]
    async fn inside_multi_threaded_runtime() {
        tokio::time::timeout(Duration::from_secs(5), async {
            JoinTasks::new()
                .and(async move { 1 })
                .and(async move { 2 })
                .join()
                .expect("could not join");
        })
        .await
        .expect("timed out");
    }
}
