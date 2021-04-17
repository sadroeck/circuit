/// Instance of a [`Proc`] which ensures all log statements are made within the provided [`Span`]
pub struct SpanProc<P>
    where
        P: Proc + Send,
{
    process: P,
    span: Span,
}

impl<P: Proc> Proc for SpanProc<P> {
    type Output = P::Output;

    fn join(&mut self) -> anyhow::Result<P::Output> {
        let Self { process, span } = self;
        span.in_scope(|| process.join())
    }

    /// Wrap this proc in a tracing span.
    fn forget(&mut self) {
        let Self { process, span } = self;
        span.in_scope(|| process.forget())
    }
}

impl<P: Proc> Drop for SpanProc<P> {
    fn drop(&mut self) {
        if self.process.join().is_err() {}
    }
}