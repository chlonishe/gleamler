//! A minimal Gleamler NIF library.
//!
//! The `#[gleam_nif]` attribute macro automatically generates the Erlang NIF
//! boilerplate (C export, arity checking, and type conversion) for each
//! function. The `init_nifs!()` macro at the bottom collects all annotated
//! functions and builds the `ErlNifEntry` table that BEAM calls on load.

use gleamler::{gleam_nif, init_nifs};

/// Adds two signed 64-bit integers.
#[gleam_nif]
fn add(a: i64, b: i64) -> i64 {
    a + b
}

/// Greets a caller by name. The `String` argument is automatically decoded
/// from an Erlang binary or list; the return value is encoded back to a binary.
#[gleam_nif]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

init_nifs!();
