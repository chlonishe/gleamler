use std::fmt;

use thiserror::Error;

use crate::codegen_runtime::{NifReturnable, NifReturned};
use crate::types::atom;
use crate::{Encoder, Env, types};

/// Represents usual errors that can happen in a nif. This enables you
/// to return an error from anywhere, even places where you don't have
/// an Env available.
#[derive(Error)]
pub enum Error {
    /// Returned when the NIF has been called with the wrong number or type of
    /// arguments.
    #[error("badarg")]
    BadArg,

    /// Encodes the string into an atom and returns it from the NIF.
    #[error("{{error, {0}}}")]
    Atom(&'static str),

    #[error("throw({0})")]
    RaiseAtom(&'static str),

    /// Encodes an arbitrary Boxed Encoder and returns it as `{error, term}`
    /// (`Error(term)` in Gleam) from the NIF. Very useful for returning
    /// descriptive, context-full errors.
    #[error("{{error, <term>}}")]
    RaiseTerm(Box<dyn Encoder>),

    #[error("{{error, <term>}}")]
    Term(Box<dyn Encoder>),

    /// NIF panicked. Carries a human-readable backtrace.
    #[error("panic({0})")]
    Panic(String),
}

impl Error {
    /// Convenience constructor for `Error::Term`.
    ///
    /// Avoids writing `Box::new(...)` at every call site.
    pub fn term<T>(value: T) -> Self
    where
        T: Encoder + 'static,
    {
        Error::Term(Box::new(value))
    }

    /// Convenience constructor for `Error::RaiseTerm`.
    pub fn raise_term<T>(value: T) -> Self
    where
        T: Encoder + 'static,
    {
        Error::RaiseTerm(Box::new(value))
    }
}

unsafe impl NifReturnable for crate::error::Error {
    unsafe fn into_returned(self, env: Env) -> NifReturned {
        match self {
            Error::BadArg => NifReturned::BadArg,
            Error::Atom(atom_str) => match types::atom::Atom::from_str(env, atom_str) {
                Ok(atom) => NifReturned::Term(atom.to_term(env).as_c_arg()),
                Err(_) => NifReturned::BadArg,
            },
            Error::RaiseAtom(atom_str) => match types::atom::Atom::from_str(env, atom_str) {
                Ok(atom) => NifReturned::Raise(atom.as_c_arg()),
                Err(_) => NifReturned::BadArg,
            },
            Error::RaiseTerm(term_unencoded) => {
                let term = term_unencoded.encode(env);
                NifReturned::Raise(term.as_c_arg())
            }
            Error::Term(term_unencoded) => {
                let term = term_unencoded.encode(env);
                let error_tuple = (atom::error(), term).encode(env);
                NifReturned::Term(error_tuple.as_c_arg())
            }
            Error::Panic(msg) => {
                let term = (atom::nif_panicked(), msg.as_str()).encode(env).as_c_arg();
                NifReturned::Raise(term)
            }
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            Error::BadArg => write!(fmt, "{{error, badarg}}"),
            Error::Atom(s) => write!(fmt, "{{error, {s}}}"),
            Error::RaiseAtom(s) => write!(fmt, "throw({s})"),
            Error::RaiseTerm(_) => write!(fmt, "throw(<term>)"),
            Error::Term(_) => write!(fmt, "{{error, <term>}}"),
            Error::Panic(s) => write!(fmt, "panic({s})"),
        }
    }
}
