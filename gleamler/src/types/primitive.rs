use crate::types::atom;
use crate::{Decoder, Encoder, Env, Error, NifResult, Term};
use std::convert::TryFrom;

macro_rules! erl_make {
    ($self:expr, $env:ident, $encode_fun:ident, $type:ty) => {
        unsafe {
            Term::new(
                $env,
                crate::sys::$encode_fun($env.as_c_arg(), $self as $type),
            )
        }
    };
}

macro_rules! erl_get {
    ($decode_fun:ident, $term:ident, $dest:ident) => {
        unsafe { crate::sys::$decode_fun($term.get_env().as_c_arg(), $term.as_c_arg(), &mut $dest) }
    };
}

macro_rules! impl_number_encoder {
    ($dec_type:ty, $nif_type:ty, $encode_fun:ident) => {
        impl Encoder for $dec_type {
            #[inline]
            fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
                erl_make!(*self, env, $encode_fun, $nif_type)
            }
        }
    };
}

macro_rules! impl_number_decoder {
    ($dec_type:ty, $nif_type:ty, $decode_fun:ident) => {
        impl<'a> Decoder<'a> for $dec_type {
            #[inline]
            fn decode(term: Term) -> NifResult<$dec_type> {
                let mut res: $nif_type = Default::default();
                if erl_get!($decode_fun, term, res) == 0 {
                    return Err(Error::BadArg);
                }
                <$dec_type>::try_from(res).map_err(|_| Error::BadArg)
            }
        }
    };
}

macro_rules! impl_number_transcoder {
    ($dec_type:ty, $nif_type:ty, $encode_fun:ident, $decode_fun:ident) => {
        impl_number_encoder!($dec_type, $nif_type, $encode_fun);
        impl_number_decoder!($dec_type, $nif_type, $decode_fun);
    };
}

macro_rules! impl_nonzero {
    ($nz:ty, $primitive:ty) => {
        impl Encoder for $nz {
            #[inline]
            fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
                self.get().encode(env)
            }
        }
        impl<'a> Decoder<'a> for $nz {
            #[inline]
            fn decode(term: Term<'a>) -> NifResult<Self> {
                let inner: $primitive = term.decode()?;
                Self::new(inner).ok_or(Error::BadArg)
            }
        }
    };
}

// Base number types
impl_number_transcoder!(i32, i32, enif_make_int, enif_get_int);
impl_number_transcoder!(u32, u32, enif_make_uint, enif_get_uint);
impl_number_transcoder!(i64, i64, enif_make_int64, enif_get_int64);
impl_number_transcoder!(u64, u64, enif_make_uint64, enif_get_uint64);
impl_number_encoder!(f64, f64, enif_make_double);

// Casted number types
impl_number_transcoder!(i8, i32, enif_make_int, enif_get_int);
impl_number_transcoder!(u8, u32, enif_make_uint, enif_get_uint);
impl_number_transcoder!(i16, i32, enif_make_int, enif_get_int);
impl_number_transcoder!(u16, u32, enif_make_uint, enif_get_uint);

// NonZero* types
impl_nonzero!(std::num::NonZeroU8, u8);
impl_nonzero!(std::num::NonZeroU16, u16);
impl_nonzero!(std::num::NonZeroU32, u32);
impl_nonzero!(std::num::NonZeroU64, u64);
impl_nonzero!(std::num::NonZeroU128, u128);
impl_nonzero!(std::num::NonZeroUsize, usize);
impl_nonzero!(std::num::NonZeroI8, i8);
impl_nonzero!(std::num::NonZeroI16, i16);
impl_nonzero!(std::num::NonZeroI32, i32);
impl_nonzero!(std::num::NonZeroI64, i64);
impl_nonzero!(std::num::NonZeroI128, i128);
impl_nonzero!(std::num::NonZeroIsize, isize);

impl Encoder for usize {
    #[inline]
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        erl_make!(*self, env, enif_make_uint64, u64)
    }
}

impl Decoder<'_> for usize {
    #[inline]
    fn decode(term: Term) -> NifResult<usize> {
        let mut res: u64 = Default::default();
        if erl_get!(enif_get_uint64, term, res) == 0 {
            return Err(Error::BadArg);
        }
        usize::try_from(res).map_err(|_| Error::BadArg)
    }
}

impl Encoder for isize {
    #[inline]
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        erl_make!(*self, env, enif_make_int64, i64)
    }
}

impl Decoder<'_> for isize {
    #[inline]
    fn decode(term: Term) -> NifResult<isize> {
        let mut res: i64 = Default::default();
        if erl_get!(enif_get_int64, term, res) == 0 {
            return Err(Error::BadArg);
        }
        isize::try_from(res).map_err(|_| Error::BadArg)
    }
}

impl_number_encoder!(f32, f64, enif_make_double);

// Manual Decoder impls for floats so they can fall back to decoding from integer terms
impl Decoder<'_> for f64 {
    fn decode(term: Term) -> NifResult<f64> {
        let mut res: f64 = Default::default();
        if erl_get!(enif_get_double, term, res) == 0 {
            return Err(Error::BadArg);
        }
        Ok(res)
    }
}

impl Decoder<'_> for f32 {
    fn decode(term: Term) -> NifResult<f32> {
        let res: f64 = term.decode()?;
        Ok(res as f32)
    }
}

impl Encoder for bool {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        if *self {
            atom::true_().to_term(env)
        } else {
            atom::false_().to_term(env)
        }
    }
}
impl<'a> Decoder<'a> for bool {
    fn decode(term: Term<'a>) -> NifResult<bool> {
        atom::decode_bool(term)
    }
}

impl Encoder for char {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        let mut buf = [0; 4];
        self.encode_utf8(&mut buf).encode(env)
    }
}

impl<'a> Decoder<'a> for char {
    fn decode(term: Term<'a>) -> NifResult<Self> {
        let s: &'a str = term.decode()?;
        let mut chars = s.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) => Ok(c),
            _ => Err(Error::BadArg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nonzero_impls_exist() {
        fn assert_encoder<T: Encoder>() {}
        fn assert_decoder<'a, T: Decoder<'a>>() {}
        assert_encoder::<std::num::NonZeroU64>();
        assert_decoder::<std::num::NonZeroU64>();
        assert_encoder::<std::num::NonZeroI32>();
        assert_decoder::<std::num::NonZeroI32>();
    }

    #[test]
    fn nonzero_decode_zero_would_fail() {
        assert!(std::num::NonZeroU64::new(0).is_none());
        assert!(std::num::NonZeroI32::new(0).is_none());
        assert_eq!(std::num::NonZeroU64::new(42).unwrap().get(), 42);
    }

    #[test]
    fn char_impls_exist() {
        fn assert_encoder<T: Encoder>() {}
        fn assert_decoder<'a, T: Decoder<'a>>() {}
        assert_encoder::<char>();
        assert_decoder::<char>();
    }

    #[test]
    fn char_decode_logic() {
        assert_eq!("a".chars().count(), 1);
        assert_eq!("é".chars().count(), 1);
        assert_eq!("".chars().count(), 0);
        assert_eq!("ab".chars().count(), 2);
    }
}
