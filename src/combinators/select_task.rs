use crate::proc::Proc;
use crate::runtime::TaskRuntime;
use std::future::IntoFuture;
use tokio::runtime::Handle;
use tokio::task::JoinHandle;

pub struct SelectTasks<T: Send + 'static> {
    runtime: TaskRuntime,
    tasks: Vec<JoinHandle<T>>,
}

impl<T: Send> Default for SelectTasks<T> {
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

impl<T: Send + 'static> SelectTasks<T> {
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
    pub fn or<F>(mut self, fut: F) -> Self
    where
        F: IntoFuture<Output = T>,
        <F as IntoFuture>::IntoFuture: Send + 'static,
    {
        self.tasks.push(Handle::current().spawn(fut.into_future()));
        self
    }
}

impl<T: Send + 'static> Proc for SelectTasks<T> {
    type Output = Option<T>;

    fn join(&mut self) -> anyhow::Result<Self::Output> {
        if self.tasks.is_empty() {
            return Ok(None);
        }

        let tasks = std::mem::take(&mut self.tasks);
        let (output_tx, output_rx) = flume::bounded(1);
        self.runtime.handle().spawn(async move {
            let (finished, _, remaining) = futures::future::select_all(tasks).await;
            let _ = output_tx.send_async((finished, remaining));
        });
        let (output, remaining) = output_rx.recv()?;
        self.tasks = remaining;
        Ok(Some(output?))
    }

    #[inline]
    fn forget(&mut self) {
        for task in self.tasks.drain(..) {
            task.abort();
        }
    }
}

/// Note: Only awaits the first ready future
impl<T: Send + 'static> Drop for SelectTasks<T> {
    fn drop(&mut self) {
        let _ = self.join();
        self.runtime.shutdown();
    }
}
