use std::sync::Mutex;

use crate::resource::{Resource, ResourceArc};
use crate::{Encoder, Env, Term};

trait ErasedIterator: Send + Sync + 'static {
    fn next_term<'a>(&self, env: Env<'a>) -> Option<Term<'a>>;
}

impl<I, T> ErasedIterator for Mutex<I>
where
    I: Iterator<Item = T> + Send + 'static,
    T: Encoder + 'static,
{
    fn next_term<'a>(&self, env: Env<'a>) -> Option<Term<'a>> {
        let mut guard = self.lock().ok()?;
        let item = guard.next()?;
        Some(item.encode(env))
    }
}

/// A BEAM resource wrapping a thread-safe Rust iterator.
pub struct Yielder {
    inner: Box<dyn ErasedIterator + Send + Sync>,
}

unsafe impl Send for Yielder {}
unsafe impl Sync for Yielder {}

impl Resource for Yielder {}

impl Yielder {
    /// Creates a new `ResourceArc<Yielder>` from any Rust iterator whose items implement `Encoder`.
    pub fn new<I, T>(iter: I) -> ResourceArc<Self>
    where
        I: Iterator<Item = T> + Send + 'static,
        T: Encoder + 'static,
    {
        ResourceArc::new(Self {
            inner: Box::new(Mutex::new(iter)),
        })
    }

    /// Fetches the next item from the iterator and encodes it into the environment.
    pub fn next<'a>(&self, env: Env<'a>) -> Option<Term<'a>> {
        self.inner.next_term(env)
    }
}
