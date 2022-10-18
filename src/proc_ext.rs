use crate::combinators::{AndThenProc, OrElseProc};
use crate::proc::Proc;

/// Extension trait for [`Proc`] allowing combinators over units of execution
pub trait ProcExt: Proc + Sized {
    fn and_then<P: Proc>(self, other: P) -> AndThenProc<Self, P>;
    fn or_else<P: Proc<Output = Self::Output>>(self, other: P) -> OrElseProc<Self, P>;
    fn boxed(self) -> Box<dyn Proc<Output = Self::Output>>;
}

impl<P: Proc + Send + 'static> ProcExt for P {
    /// Similar to [`Result::and`] but with procs.
    fn and_then<O: Proc>(self, other: O) -> AndThenProc<P, O> {
        AndThenProc {
            left: self,
            right: other,
        }
    }

    /// Similar to [`Result::or_else`] but with procs.
    fn or_else<O: Proc<Output = P::Output>>(self, other: O) -> OrElseProc<P, O> {
        OrElseProc {
            left: self,
            right: other,
        }
    }

    fn boxed(self) -> Box<dyn Proc<Output = Self::Output>> {
        Box::new(self)
    }
}
