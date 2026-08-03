// #![allow(unsafe_op_in_unsafe_fn, dead_code, unused_imports)]

extern crate self as gleamler;

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
mod r#return;
mod schedule;
mod thread;
mod nif;

pub use crate::alloc::EnifAllocator;
pub use crate::dynamic::TermType;
pub use crate::env::{Env, OwnedEnv};
pub use crate::error::Error;
pub use crate::nif::Nif;
pub use crate::r#return::Return;
pub use crate::resource::{Monitor, Resource, ResourceArc, ResourceInitError};
pub use crate::schedule::SchedulerFlags;
pub use crate::term::Term;
pub use crate::thread::{spawn, JobSpawner, ThreadSpawner};
pub use crate::types::{
    Atom, Binary, Decoder, Encoder, ErlOption, ListIterator, LocalPid, MapIterator, NewBinary,
    OwnedBinary, Reference,
};

#[macro_use]
pub mod types;

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

#[gleam_nif]
pub fn add(a: i64, b: i64) -> i64 {
    a + b
}

#[gleam_nif]
pub fn sub(a: i64, b: i64) -> i64 {
    a - b
}

#[gleam_nif]
pub fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

#[gleam_nif]
pub fn double_list(items: Vec<i64>) -> Vec<i64> {
    items.into_iter().map(|x| x * 2).collect()
}

#[gleam_nif]
pub fn is_positive(n: i64) -> bool {
    n > 0
}

#[gleam_nif]
pub fn divide(a: f64, b: f64) -> f64 {
    a / b
}

#[gleam_nif]
pub fn make_pair(a: i64, b: String) -> (i64, String) {
    (a, b)
}

init_nifs!([add, sub, greet, double_list, is_positive, divide, make_pair]);