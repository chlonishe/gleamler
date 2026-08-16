use crate::{Env, Error, NifResult, Term};

#[macro_use]
pub mod atom;
pub use crate::types::atom::Atom;

pub mod binary;
pub use crate::types::binary::{Binary, NewBinary, OwnedBinary};

#[cfg(feature = "big_integer")]
pub mod big_int;
#[cfg(feature = "big_integer")]
pub use num_bigint::BigInt;

#[doc(hidden)]
pub mod list;
pub use crate::types::list::ListIterator;

#[doc(hidden)]
pub mod map;
pub use self::map::MapIterator;

#[doc(hidden)]
pub mod primitive;
#[doc(hidden)]
pub mod string;
pub mod tuple;

#[doc(hidden)]
pub mod local_pid;
pub use self::local_pid::LocalPid;

#[doc(hidden)]
pub mod reference;
pub use self::reference::Reference;

pub mod i128;
pub mod path;

pub mod truthy;

pub mod elixir_struct;

pub mod erlang_option;
pub use self::erlang_option::ErlOption;

pub trait Encoder {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a>;
}
pub trait Decoder<'a>: Sized + 'a {
    fn decode(term: Term<'a>) -> NifResult<Self>;
}

impl Encoder for Term<'_> {
    fn encode<'b>(&self, env: Env<'b>) -> Term<'b> {
        self.in_env(env)
    }
}
impl<'a> Decoder<'a> for Term<'a> {
    fn decode(term: Term<'a>) -> NifResult<Self> {
        Ok(term)
    }
}

impl<T> Encoder for &T
where
    T: Encoder,
{
    fn encode<'c>(&self, env: Env<'c>) -> Term<'c> {
        <T as Encoder>::encode(self, env)
    }
}

impl<T> Encoder for Box<T>
where
    T: Encoder,
{
    fn encode<'c>(&self, env: Env<'c>) -> Term<'c> {
        self.as_ref().encode(env)
    }
}

impl<'a, T> Decoder<'a> for Box<T>
where
    T: Decoder<'a>,
{
    fn decode(term: Term<'a>) -> NifResult<Self> {
        term.decode().map(Box::new)
    }
}

impl<T> Encoder for Option<T>
where
    T: Encoder,
{
    fn encode<'c>(&self, env: Env<'c>) -> Term<'c> {
        match *self {
            Some(ref value) => (atom::some(), value.encode(env)).encode(env),
            None => atom::none().encode(env),
        }
    }
}

impl<'a, T> Decoder<'a> for Option<T>
where
    T: Decoder<'a>,
{
    fn decode(term: Term<'a>) -> NifResult<Self> {
        if let Ok((tag, inner)) = term.decode::<(atom::Atom, Term<'a>)>() {
            let tag_str = tag.to_term(term.get_env()).atom_to_string().unwrap_or_default();
            if tag == atom::some() || tag_str == "Some" {
                return Ok(Some(inner.decode()?));
            }
        }
        if let Ok(decoded_atom) = term.decode::<atom::Atom>() {
            let s = term.atom_to_string().unwrap_or_default();
            if decoded_atom == atom::none()
                || decoded_atom == atom::nil()
                || s == "None"
            {
                return Ok(None);
            }
        }
        Err(Error::BadArg)
    }
}

impl<T, E> Encoder for Result<T, E>
where
    T: Encoder,
    E: Encoder,
{
    fn encode<'c>(&self, env: Env<'c>) -> Term<'c> {
        match *self {
            Ok(ref value) => (atom::ok().encode(env), value.encode(env)).encode(env),
            Err(ref err) => (atom::error().encode(env), err.encode(env)).encode(env),
        }
    }
}

impl<'a, T, E> Decoder<'a> for Result<T, E>
where
    T: Decoder<'a>,
    E: Decoder<'a>,
{
    fn decode(term: Term<'a>) -> NifResult<Self> {
        let (decoded_atom, inner_term): (atom::Atom, Term) = term.decode()?;
        if decoded_atom == atom::ok() {
            let ok_value: T = inner_term.decode()?;
            Ok(Ok(ok_value))
        } else if decoded_atom == atom::error() {
            let err_value: E = inner_term.decode()?;
            Ok(Err(err_value))
        } else {
            Err(Error::BadArg)
        }
    }
}

impl<'a, K, V> Decoder<'a> for std::collections::HashMap<K, V>
where
    K: Decoder<'a> + Eq + std::hash::Hash,
    V: Decoder<'a>,
{
    fn decode(term: Term<'a>) -> NifResult<Self> {
        let size = term.map_size()?;

        let it = MapIterator::new(term).ok_or(Error::BadArg)?;

        let mut map = std::collections::HashMap::with_capacity(size);

        for (k, v) in it {
            let k = k.decode()?;
            let v = v.decode()?;
            map.insert(k, v);
        }

        Ok(map)
    }
}

impl<K, V> Encoder for std::collections::HashMap<K, V>
where
    K: Encoder + Eq + std::hash::Hash,
    V: Encoder,
{
    fn encode<'c>(&self, env: Env<'c>) -> Term<'c> {
        let (keys, values): (Vec<_>, Vec<_>) = self.iter().unzip();
        Term::map_from_arrays(env, &keys, &values).unwrap()
    }
}

impl Encoder for () {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        atom::nil().to_term(env)
    }
}

impl<'a> Decoder<'a> for () {
    fn decode(term: Term<'a>) -> NifResult<Self> {
        if term.is_empty_list() || term == atom::nil().to_term(term.get_env()) {
            Ok(())
        } else {
            Err(Error::BadArg)
        }
    }
}