mod combinators;
mod proc;
mod proc_ext;
mod runners;

pub use crate::combinators::*;
pub use crate::runners::*;
pub use proc::Proc;
pub use proc_ext::ProcExt;

/// Execute a future to completion using a tokio current-thread scheduler.
#[cfg(feature = "tokio")]
pub fn tokio<T: Send>(
    fut: impl std::future::Future<Output = anyhow::Result<T>> + Send + 'static,
) -> BlockingProc<impl FnOnce() -> anyhow::Result<T>, T> {
    blocking(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        rt.block_on(fut)
    })
}

/// Execute a future to completion using a [`smol::LocalExecutor`].
// #[cfg(feature = "smol")]
// pub fn smol<T: Send>(
//     fut: impl std::future::Future<Output = anyhow::Result<T>> + Send + 'static,
// ) -> BlockingProc<impl FnOnce() -> anyhow::Result<T>, T> {
//     blocking(move || {
//         let executor = smol::LocalExecutor::default();
//         smol::block_on(executor.run(fut))
//     })
// }

/// Executes a function to completion using a blocking call
pub fn blocking<F, T>(f: F) -> BlockingProc<F, T>
where
    F: FnOnce() -> anyhow::Result<T> + Send,
    T: Send,
{
    BlockingProc(Some(f))
}

/// Executes a function to completion on a native OS thread
pub fn thread<F, T>(f: F) -> NativeThread<T>
where
    F: FnOnce() -> anyhow::Result<T> + Send + 'static,
    T: Send + 'static,
{
    NativeThread(Some(std::thread::spawn(f)))
}

/// Immediately returns without executing.
/// Useful for building recursive [`Proc`] chains
pub fn nop() -> NopProc {
    NopProc {}
}
