#![allow(dead_code)]

use crate::serde::error::Error;
use crate::types::atom::Atom;
use crate::{Env, Term};

crate::atoms! {
    nil,
    true_ = "true",
    false_ = "false",
    some,
    none,
    ok,
    error,
    nan,
    inf,
    neg_inf,
    __struct__ = "__struct__",
}

pub fn term_to_string(term: &Term) -> Result<String, Error> {
    term.atom_to_string().map_err(|_| Error::InvalidAtom)
}

pub fn str_to_term<'a>(env: Env<'a>, s: &str) -> Result<Term<'a>, Error> {
    Atom::from_str(env, s)
        .map(|a| a.to_term(env))
        .map_err(|_| Error::InvalidAtom)
}
