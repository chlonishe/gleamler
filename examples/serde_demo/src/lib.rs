//! Serde integration example.
//!
//! Enable the `"serde"` feature in `Cargo.toml`:
//!
//! ```toml
//! gleamler = { path = "../../gleamler", features = ["serde"] }
//! ```
//!
//! `from_term` deserializes any Erlang term into a Rust type.
//! `to_term` serializes a Rust type back into an Erlang term.
use gleamler::serde::{from_term, to_term};
use gleamler::{Env, Error, NifResult, Term, gleam_nif, init_nifs};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct Point {
    x: f64,
    y: f64,
}

/// Decodes a term into a `Point` and returns its debug representation.
/// Errors are mapped to `{error, Reason}` tuples in Erlang.
#[gleam_nif]
fn inspect_point(term: Term) -> NifResult<String> {
    let p: Point = from_term(term).map_err(|e| Error::term(e.to_string()))?;
    Ok(format!("{:?}", p))
}

/// Builds an Erlang term from a Rust struct.
#[gleam_nif]
fn make_point(env: Env, x: f64, y: f64) -> NifResult<Term> {
    let p = Point { x, y };
    to_term(env, p).map_err(|e| Error::term(e.to_string()))
}

init_nifs!();
