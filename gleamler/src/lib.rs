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
mod term;
#[cfg(test)]
mod tests;
mod thread;

pub mod cancellation;
pub mod log;
#[cfg(any(feature = "nifs", feature = "stress", test))]
pub mod nifs;
pub mod resource;
pub mod schedule;
#[cfg(feature = "stress")]
pub mod stress_nifs;
pub mod types;
pub mod yielder;

#[cfg(feature = "serde")]
pub mod serde;
#[cfg(feature = "serde")]
pub use crate::serde::SerdeTerm;

pub use crate::alloc::EnifAllocator;
pub use crate::cancellation::{CancellationResource, CancellationToken};
pub use crate::codegen_runtime::NifOutcome;
pub use crate::dynamic::TermType;
#[cfg(feature = "nif_version_2_17")]
pub use crate::env::NifOption;
pub use crate::env::UniqueIntegerFlags;
pub use crate::env::{Env, OwnedEnv};
pub use crate::error::{Error, GleamlerError};
pub use crate::log::Level as LogLevel;
pub use crate::nif::Nif;
pub use crate::resource::{Monitor, Resource, ResourceArc, ResourceInitError};
pub use crate::schedule::SelectFlags;
pub use crate::schedule::{SchedulerFlags, consume_timeslice};
pub use crate::term::Term;
pub use crate::thread::{JobSpawner, ThreadSpawner, spawn};
pub use crate::types::{
    Atom, Binary, BitArray, Decoder, Encoder, ErlOption, ListIterator, LocalPid, LocalPort,
    MapIterator, NewBinary, OwnedBinary, Reference, SavedSubject, Subject, SubjectSender,
};
pub use crate::yielder::Yielder;

pub type NifResult<T> = Result<T, Error>;

pub use gleamler_macros::{
    NifMap, NifRecord, NifTaggedEnum, NifTuple, NifUnitEnum, NifUntaggedEnum, gleam_nif, init_nifs,
    resource_impl,
};

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
