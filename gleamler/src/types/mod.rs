use crate::{Env, Error, NifResult, Term};

#[macro_use]
pub mod atom;
pub use crate::types::atom::Atom;

pub mod binary;
pub use crate::types::binary::{Binary, NewBinary, OwnedBinary};

pub mod duration;
pub use self::duration::ErlangTimestamp;

pub mod collections;
pub mod net;
pub mod time;

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

#[doc(hidden)]
pub mod local_port;
pub use self::local_port::LocalPort;

pub mod i128;
pub mod path;

pub mod erlang_option;
pub use self::erlang_option::ErlOption;

pub mod subject;
pub use self::subject::Subject;

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
        if let Ok((tag, inner)) = term.decode::<(atom::Atom, Term<'a>)>()
            && tag == atom::some()
        {
            return Ok(Some(inner.decode()?));
        }
        if let Ok(decoded_atom) = term.decode::<atom::Atom>()
            && (decoded_atom == atom::none() || decoded_atom == atom::nil())
        {
            return Ok(None);
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
        let mut keys = Vec::with_capacity(self.len());
        let mut values = Vec::with_capacity(self.len());
        for (k, v) in self {
            keys.push(k.encode(env).as_c_arg());
            values.push(v.encode(env).as_c_arg());
        }
        Term::map_from_raw_arrays(env, &keys, &values)
            .expect("enif_make_map_from_arrays failed (OOM or duplicate keys)")
    }
}

impl<'a, K, V> Decoder<'a> for std::collections::BTreeMap<K, V>
where
    K: Decoder<'a> + Ord,
    V: Decoder<'a>,
{
    fn decode(term: Term<'a>) -> NifResult<Self> {
        let it = MapIterator::new(term).ok_or(Error::BadArg)?;
        let mut map = std::collections::BTreeMap::new();
        for (k, v) in it {
            map.insert(k.decode()?, v.decode()?);
        }
        Ok(map)
    }
}

impl<K, V> Encoder for std::collections::BTreeMap<K, V>
where
    K: Encoder + Ord,
    V: Encoder,
{
    fn encode<'c>(&self, env: Env<'c>) -> Term<'c> {
        let mut keys = Vec::with_capacity(self.len());
        let mut values = Vec::with_capacity(self.len());
        for (k, v) in self {
            keys.push(k.encode(env).as_c_arg());
            values.push(v.encode(env).as_c_arg());
        }
        Term::map_from_raw_arrays(env, &keys, &values)
            .expect("enif_make_map_from_arrays failed (OOM or duplicate keys)")
    }
}

impl Encoder for () {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        unsafe { Term::new(env, crate::wrapper::list::make_list(env.as_c_arg(), &[])) }
    }
}

impl<'a> Decoder<'a> for () {
    fn decode(term: Term<'a>) -> NifResult<Self> {
        if term.is_empty_list() {
            Ok(())
        } else {
            Err(Error::BadArg)
        }
    }
}

#[cfg(feature = "uuid")]
impl Encoder for uuid::Uuid {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        let bytes = self.as_bytes();
        let mut bin = crate::NewBinary::new(env, bytes.len());
        bin.as_mut_slice().copy_from_slice(bytes);
        bin.into()
    }
}

#[cfg(feature = "uuid")]
impl<'a> Decoder<'a> for uuid::Uuid {
    fn decode(term: Term<'a>) -> NifResult<Self> {
        let bin = crate::Binary::from_term(term)?;
        uuid::Uuid::from_slice(bin.as_slice()).map_err(|_| Error::BadArg)
    }
}

#[cfg(feature = "rust_decimal")]
impl Encoder for rust_decimal::Decimal {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        self.to_string().encode(env)
    }
}

#[cfg(feature = "rust_decimal")]
impl<'a> Decoder<'a> for rust_decimal::Decimal {
    fn decode(term: Term<'a>) -> NifResult<Self> {
        let s: String = term.decode()?;
        s.parse().map_err(|_| Error::BadArg)
    }
}

#[cfg(feature = "bytes")]
impl Encoder for bytes::Bytes {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        let mut bin = crate::NewBinary::new(env, self.len());
        bin.as_mut_slice().copy_from_slice(self);
        bin.into()
    }
}

#[cfg(feature = "bytes")]
impl<'a> Decoder<'a> for bytes::Bytes {
    fn decode(term: Term<'a>) -> NifResult<Self> {
        let bin = crate::Binary::from_term(term)?;
        Ok(bytes::Bytes::copy_from_slice(bin.as_slice()))
    }
}

#[cfg(test)]
mod type_tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn btree_map_trait_impls_exist() {
        fn assert_enc<T: Encoder>() {}
        fn assert_dec<'a, T: Decoder<'a>>() {}

        assert_enc::<BTreeMap<String, i64>>();
        assert_dec::<BTreeMap<String, i64>>();
        assert_enc::<BTreeMap<i64, String>>();
        assert_dec::<BTreeMap<i64, String>>();
    }
}
