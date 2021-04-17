use std::ops::DerefMut;

/// Callable unit of execution. Similar to [`std::thread`] but enforces join-on-drop, supports
/// combinators, and allows foreground execution.
pub trait Proc: Send {
    type Output: Send;
    fn join(&mut self) -> anyhow::Result<Self::Output>;
    fn forget(&mut self);
}

impl<P: Proc> Proc for Box<P> {
    type Output = P::Output;

    fn join(&mut self) -> anyhow::Result<Self::Output> {
        self.deref_mut().join()
    }

    fn forget(&mut self) {
        self.deref_mut().forget()
    }
}

impl<T: Send> Proc for Box<dyn Proc<Output = T>> {
    type Output = T;

    fn join(&mut self) -> anyhow::Result<Self::Output> {
        self.deref_mut().join()
    }

    fn forget(&mut self) {
        self.deref_mut().forget()
    }
}