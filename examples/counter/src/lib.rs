//! Resource example: mutable state living in the BEAM heap.
//!
//! A `Resource` is a Rust struct managed by the Erlang GC. Before you can
//! create `ResourceArc<Counter>` from a NIF, the type must be registered
//! during `on_load`. Without `env.register::<Counter>()`,
//! `ResourceArc::new` will panic at runtime.
use gleamler::{Env, Resource, ResourceArc, Term, gleam_nif, init_nifs};
use std::sync::atomic::{AtomicI64, Ordering};

/// Counter holds mutable state that survives across NIF calls.
pub struct Counter {
    value: AtomicI64,
}

impl Resource for Counter {}

/// Creates a new counter initialized to zero.
#[gleam_nif]
fn counter_new() -> ResourceArc<Counter> {
    ResourceArc::new(Counter {
        value: AtomicI64::new(0),
    })
}

/// Atomically increments the counter and returns the *new* value.
/// `fetch_add` returns the previous value, hence the `+ 1`.
#[gleam_nif]
fn counter_inc(counter: ResourceArc<Counter>) -> i64 {
    counter.value.fetch_add(1, Ordering::SeqCst) + 1
}

/// Returns the current value.
#[gleam_nif]
fn counter_get(counter: ResourceArc<Counter>) -> i64 {
    counter.value.load(Ordering::SeqCst)
}

/// Resource registration is mandatory. It tells the BEAM VM about the
/// type's size, destructor, and optional callbacks (e.g. `down`).
fn on_load(env: Env, _info: Term) -> bool {
    env.register::<Counter>().is_ok()
}

init_nifs!(load = on_load);
