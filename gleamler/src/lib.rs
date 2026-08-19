extern crate self as gleamler;

#[cfg(feature = "allocator")]
#[global_allocator]
static GLOBAL_ALLOCATOR: crate::alloc::EnifAllocator = crate::alloc::EnifAllocator;

#[doc(hidden)]
pub mod wrapper;
#[doc(hidden)]
pub mod codegen_runtime;
#[doc(hidden)]
pub mod sys;

mod alloc;
mod term;
mod resource;
mod dynamic;
mod env;
mod error;
mod schedule;
mod thread;
mod nif;
#[cfg(test)]
mod tests;

pub mod nifs;
pub mod types;

pub use crate::alloc::EnifAllocator;
pub use crate::dynamic::TermType;
pub use crate::env::{Env, OwnedEnv};
pub use crate::error::Error;
pub use crate::nif::Nif;
pub use crate::resource::{Monitor, Resource, ResourceArc, ResourceInitError};
pub use crate::schedule::{consume_timeslice, SchedulerFlags};
pub use crate::term::Term;
pub use crate::thread::{spawn, JobSpawner, ThreadSpawner};
pub use crate::types::{
    Atom, Binary, Decoder, Encoder, ErlOption, ListIterator, LocalPid, MapIterator, NewBinary,
    OwnedBinary, Reference,
};

pub type NifResult<T> = Result<T, Error>;

pub use gleamler_macros::{gleam_nif, init_nifs};

#[macro_export]
macro_rules! term_map {
    ($env:expr, { $($key:expr => $value:expr),* $(,)? }) => {{
        $crate::Term::map_from_term_arrays(
            $env,
            &[$($crate::Encoder::encode(&$key, $env)),*],
            &[$($crate::Encoder::encode(&$value, $env)),*],
        )
        .expect("failed to create map")
    }};
}
