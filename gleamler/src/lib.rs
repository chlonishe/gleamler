extern crate self as gleamler;

#[cfg(feature = "allocator")]
#[global_allocator]
static GLOBAL_ALLOCATOR: crate::alloc::EnifAllocator = crate::alloc::EnifAllocator;

#[doc(hidden)]
pub mod codegen_runtime;
#[doc(hidden)]
pub mod sys;
#[doc(hidden)]
pub mod wrapper;

mod alloc;
mod dynamic;
mod env;
mod error;
mod nif;
mod resource;
mod schedule;
mod term;
#[cfg(test)]
mod tests;
mod thread;

pub mod nifs;
#[cfg(feature = "stress")]
pub mod stress_nifs;
pub mod types;

pub use crate::env::UniqueIntegerFlags;
#[cfg(feature = "nif_version_2_17")]
pub use crate::env::NifOption;
pub use crate::schedule::SelectFlags;
pub use crate::alloc::EnifAllocator;
pub use crate::dynamic::TermType;
pub use crate::env::{Env, OwnedEnv};
pub use crate::error::Error;
pub use crate::nif::Nif;
pub use crate::resource::{Monitor, Resource, ResourceArc, ResourceInitError};
pub use crate::schedule::{SchedulerFlags, consume_timeslice};
pub use crate::term::Term;
pub use crate::thread::{JobSpawner, ThreadSpawner, spawn};
pub use crate::types::{
    Atom, Binary, Decoder, Encoder, ErlOption, ListIterator, LocalPid, LocalPort,
    MapIterator, NewBinary, OwnedBinary, Reference,
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
    }};
}
