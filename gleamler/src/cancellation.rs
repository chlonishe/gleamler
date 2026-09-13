use std::sync::atomic::{AtomicBool, Ordering};

use crate::resource::{Monitor, Resource, ResourceArc};
use crate::{Decoder, Encoder, Env, Error, LocalPid, NifResult, Term};

pub struct CancellationResource {
    cancelled: AtomicBool,
}

impl Resource for CancellationResource {
    const IMPLEMENTS_DOWN: bool = true;

    fn down<'a>(&'a self, _env: Env<'a>, _pid: LocalPid, _monitor: Monitor) {
        self.cancelled.store(true, Ordering::SeqCst);
    }
}

#[derive(Clone)]
pub struct CancellationToken {
    resource: ResourceArc<CancellationResource>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self {
            resource: ResourceArc::new(CancellationResource {
                cancelled: AtomicBool::new(false),
            }),
        }
    }

    pub fn for_caller(env: Env) -> NifResult<Self> {
        let token = Self::new();
        let pid = env.pid();
        token.resource.monitor(env, &pid).ok_or(Error::BadArg)?;
        Ok(token)
    }

    #[inline]
    pub fn is_cancelled(&self) -> bool {
        self.resource.cancelled.load(Ordering::Relaxed)
    }

    #[inline]
    pub fn cancel(&self) {
        self.resource.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn as_resource(&self) -> &ResourceArc<CancellationResource> {
        &self.resource
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

impl Encoder for CancellationToken {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        self.resource.encode(env)
    }
}

impl<'a> Decoder<'a> for CancellationToken {
    fn decode(term: Term<'a>) -> NifResult<Self> {
        let resource: ResourceArc<CancellationResource> = term.decode()?;
        Ok(Self { resource })
    }
}
