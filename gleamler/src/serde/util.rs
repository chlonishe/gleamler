use crate::serde::atoms;
use crate::serde::error::Error;
use crate::{Binary, Decoder, Term, types::tuple};

pub fn term_to_str(term: &Term) -> Result<String, Error> {
    if let Ok(s) = atoms::term_to_string(term) {
        return Ok(s);
    }
    if let Ok(s) = term.decode::<String>() {
        return Ok(s);
    }
    Err(Error::ExpectedStringable)
}

pub fn is_nil(term: &Term) -> bool {
    atoms::nil().eq(term)
}

pub fn is_none(term: &Term) -> bool {
    atoms::none().eq(term)
}

pub fn parse_bool(term: &Term) -> Result<bool, Error> {
    if atoms::true_().eq(term) {
        Ok(true)
    } else if atoms::false_().eq(term) {
        Ok(false)
    } else {
        Err(Error::ExpectedBoolean)
    }
}

pub fn parse_binary<'a>(term: Term<'a>) -> Result<&'a [u8], Error> {
    validate_binary(&term)?;
    let binary: Binary = term.decode().map_err(|_| Error::ExpectedBinary)?;
    Ok(binary.as_slice())
}

pub fn parse_number<'a, T: Decoder<'a>>(term: &Term<'a>) -> Result<T, Error> {
    if !term.is_number() {
        return Err(Error::InvalidNumber);
    }
    term.decode().map_err(|_| Error::ExpectedNumber)
}

pub fn parse_str<'a>(term: Term<'a>) -> Result<&'a str, Error> {
    let bytes = parse_binary(term)?;
    std::str::from_utf8(bytes).map_err(|_| Error::ExpectedStringable)
}

pub fn validate_binary(term: &Term) -> Result<(), Error> {
    if !term.is_binary() {
        Err(Error::ExpectedBinary)
    } else {
        Ok(())
    }
}

pub fn validate_tuple(term: Term, len: Option<usize>) -> Result<Vec<Term>, Error> {
    if !term.is_tuple() {
        return Err(Error::ExpectedTuple);
    }
    let tuple = tuple::get_tuple(term).map_err(|_| Error::ExpectedTuple)?;
    match len {
        None => Ok(tuple),
        Some(len) => {
            if tuple.len() == len {
                Ok(tuple)
            } else {
                Err(Error::InvalidTuple)
            }
        }
    }
}
