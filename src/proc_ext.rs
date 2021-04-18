use crate::combinators::BlockingProc;
use crate::proc::Proc;

/// Extension trait for [`Proc`] allowing combinators over units of execution
pub trait ProcExt: Proc + Sized {
    fn and<P: Proc>(self, other: P) -> AndProc<Self, P>;
    fn run_before<F: FnOnce() -> anyhow::Result<()> + Send + 'static>(
        self,
        f: F,
    ) -> AndProc<Self, BlockingProc<F, ()>>;
    fn run_after<F: FnOnce() -> anyhow::Result<()> + Send + 'static>(
        self,
        f: F,
    ) -> AndProc<BlockingProc<F, ()>, Self>;
    fn boxed(self) -> Box<dyn Proc<Output = Self::Output>>;
    #[cfg(feature = "span")]
    fn in_span(self, span: Span) -> SpanProc<Self>;
}

/// [`Proc`] combinator that allows combining the results of two units of execution
/// sharing the same result types
pub struct AndProc<L, R>
where
    L: Proc + Send,
    R: Proc + Send,
{
    left: L,
    right: R,
}

impl<L, R> Proc for AndProc<L, R>
where
    L: Proc + Send,
    R: Proc + Send,
{
    type Output = R::Output;

    fn join(&mut self) -> anyhow::Result<Self::Output> {
        self.left.join().and(self.right.join())
    }

    fn forget(&mut self) {
        self.left.forget();
        self.right.forget();
    }
}

impl<P: Proc + Send + 'static> ProcExt for P {
    /// Similar to [`Result::and`] but with procs.
    fn and<O: Proc>(self, other: O) -> AndProc<P, O> {
        AndProc {
            left: self,
            right: other,
        }
    }

    /// Execute `self` before the provided function `f`.
    fn run_before<F: FnOnce() -> anyhow::Result<()> + Send + 'static>(
        self,
        f: F,
    ) -> AndProc<Self, BlockingProc<F, ()>> {
        self.and(BlockingProc(Some(f)))
    }

    /// Execute `self` after the provided function `f`. Useful for enforcing graceful shutdown.
    fn run_after<F: FnOnce() -> anyhow::Result<()> + Send + 'static>(
        self,
        f: F,
    ) -> AndProc<BlockingProc<F, ()>, Self> {
        BlockingProc(Some(f)).and(self)
    }

    fn boxed(self) -> Box<dyn Proc<Output = Self::Output>> {
        Box::new(self)
    }

    #[cfg(feature = "span")]
    fn in_span(self, span: Span) -> SpanProc<Self> {
        SpanProc {
            process: self,
            span,
        }
    }
}
