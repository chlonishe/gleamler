use std::fmt;

use thiserror::Error;

use crate::codegen_runtime::{NifReturnable, NifReturned};
use crate::types::atom;
use crate::types::atom::Atom;
use crate::{Decoder, Encoder, Env, NifResult, Term, types};

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
    #[error("throw(<term>)")]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GleamlerError {
    BadArg,
    Panic(String),
    Custom(String),
}

impl Encoder for GleamlerError {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        match self {
            GleamlerError::BadArg => Atom::from_str(env, "bad_arg").unwrap().to_term(env),
            GleamlerError::Panic(msg) => {
                let tag = Atom::from_str(env, "panic").unwrap();
                (tag, msg.as_str()).encode(env)
            }
            GleamlerError::Custom(msg) => {
                let tag = Atom::from_str(env, "custom").unwrap();
                (tag, msg.as_str()).encode(env)
            }
        }
    }
}

impl<'a> Decoder<'a> for GleamlerError {
    fn decode(term: Term<'a>) -> NifResult<Self> {
        if let Ok(atom) = term.decode::<Atom>()
            && let Ok(s) = atom.to_term(term.get_env()).atom_to_string()
            && (s == "bad_arg" || s == "badarg")
        {
            return Ok(GleamlerError::BadArg);
        }
        if let Ok((tag, msg)) = term.decode::<(Atom, String)>()
            && let Ok(s) = tag.to_term(term.get_env()).atom_to_string()
        {
            if s == "panic" {
                return Ok(GleamlerError::Panic(msg));
            } else if s == "custom" {
                return Ok(GleamlerError::Custom(msg));
            }
        }
        Err(Error::BadArg)
    }
}
