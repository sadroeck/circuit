use std::thread::JoinHandle;
use crate::proc::Proc;

/// Instance of a [`Proc`] which runs offloads a callable into a native OS thread
pub struct NativeThread<T: Send>(Option<JoinHandle<anyhow::Result<T>>>);

impl<T: Send> Proc for NativeThread<T> {
    type Output = T;

    fn join(&mut self) -> anyhow::Result<Self::Output> {
        if let Some(t) = self.0.take() {
            t.join()
                .map_err(|_err| anyhow::anyhow!("Could not join error"))?
        } else {
            Err(anyhow::anyhow!("Nothing to join"))
        }
    }

    fn forget(&mut self) {}
}

impl<T: Send> Drop for NativeThread<T> {
    fn drop(&mut self) {
        if self.join().is_err() {}
    }
}