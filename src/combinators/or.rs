use crate::proc::Proc;

/// [`Proc`] combinator that allows combining the results of two units of execution
/// sharing the same result types
pub struct OrElseProc<L, R>
where
    L: Proc + Send,
    R: Proc<Output = L::Output> + Send,
{
    pub(crate) left: L,
    pub(crate) right: R,
}

impl<L, R> Proc for OrElseProc<L, R>
where
    L: Proc + Send,
    R: Proc<Output = L::Output> + Send,
{
    type Output = R::Output;

    fn join(&mut self) -> anyhow::Result<Self::Output> {
        self.left.join().or_else(|_| self.right.join())
    }

    fn forget(&mut self) {
        self.left.forget();
        self.right.forget();
    }
}

impl<L, R> Drop for OrElseProc<L, R>
where
    L: Proc + Send,
    R: Proc<Output = L::Output> + Send,
{
    fn drop(&mut self) {
        let _ = self.join();
        self.forget();
    }
}

#[cfg(test)]
mod test {
    use crate::blocking;
    use crate::proc::Proc;
    use crate::proc_ext::ProcExt;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;

    fn add_to_counter(counter: Arc<AtomicU64>, val: u64) -> impl Proc {
        blocking(move || Ok(counter.fetch_add(val, Ordering::SeqCst)))
    }

    #[test]
    fn join_on_drop() {
        let counter = Arc::new(AtomicU64::new(0));
        let left = add_to_counter(counter.clone(), 1);
        let right = add_to_counter(counter.clone(), 2);
        let joined = left.or_else(right);
        drop(joined);
        assert_eq!(counter.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn forget() {
        let counter = Arc::new(AtomicU64::new(0));
        let left = add_to_counter(counter.clone(), 1);
        let right = add_to_counter(counter.clone(), 2);
        left.or_else(right).forget();
        assert_eq!(counter.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn nested() {
        let counter = Arc::new(AtomicU64::new(0));
        let left = add_to_counter(counter.clone(), 1);
        let middle = add_to_counter(counter.clone(), 2);
        let right = blocking(|| {
            add_to_counter(counter.clone(), 4)
                .and_then(add_to_counter(counter.clone(), 8))
                .join()
        });
        let joined = left.or_else(middle).or_else(right);
        drop(joined);
        assert_eq!(counter.load(Ordering::Relaxed), 1);
    }
}
