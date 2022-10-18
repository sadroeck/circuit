mod and;
mod or;

pub use and::AndThenProc;
pub use or::OrElseProc;

use crate::proc::Proc;

/// Instance of a [`Proc`] which calls a simple function
pub struct BlockingProc<F, T>(pub(crate) Option<F>)
where
    F: FnOnce() -> anyhow::Result<T> + Send,
    T: Send;

impl<F, T> Proc for BlockingProc<F, T>
where
    F: FnOnce() -> anyhow::Result<T> + Send,
    T: Send,
{
    type Output = T;

    fn join(&mut self) -> anyhow::Result<T> {
        if let Some(f) = self.0.take() {
            f()
        } else {
            Err(anyhow::Error::msg("Nothing to join"))
        }
    }

    fn forget(&mut self) {
        self.0.take();
    }
}

impl<F, T> Drop for BlockingProc<F, T>
where
    F: FnOnce() -> anyhow::Result<T> + Send,
    T: Send,
{
    fn drop(&mut self) {
        if self.join().is_err() {}
    }
}

/// Instance of a [`Proc`] which returns immediately when called
pub struct NopProc;

impl Proc for NopProc {
    type Output = ();

    fn join(&mut self) -> anyhow::Result<()> {
        Ok(())
    }

    fn forget(&mut self) {}
}
