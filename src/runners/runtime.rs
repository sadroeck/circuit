use flume::Sender;
use std::thread;
use tokio::runtime::Handle;

/// [`TaskRuntime`] is a runtime for a set of tasks that is either dedicated for a set of tasks
/// or derived from the currently active runtime
pub enum TaskRuntime {
    Owned {
        handle: Handle,
        shutdown: Sender<()>,
    },
    Entered(Handle),
}

impl TaskRuntime {
    pub fn new() -> Self {
        let (runtime_tx, runtime_rx) = flume::bounded(1);
        thread::spawn(move || {
            // Create a shutdown signal
            let (shutdown_tx, shutdown_rx) = flume::bounded(1);

            // Create a new single-threaded runtime
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            // Share the handle
            runtime_tx
                .send(Self::Owned {
                    handle: runtime.handle().clone(),
                    shutdown: shutdown_tx,
                })
                .unwrap();

            // Allow tokio::spawn within the new context
            let _ = runtime.enter();

            // Wait for the shutdown signal
            let _ = runtime.block_on(shutdown_rx.recv_async());
        });
        runtime_rx.recv().unwrap()
    }

    pub fn handle(&self) -> &Handle {
        match self {
            TaskRuntime::Owned { handle, .. } => handle,
            TaskRuntime::Entered(handle) => handle,
        }
    }

    pub fn shutdown(&mut self) {
        match self {
            TaskRuntime::Owned { shutdown, .. } => {
                let _ = shutdown.send(());
            }
            TaskRuntime::Entered(_) => {}
        }
    }
}
