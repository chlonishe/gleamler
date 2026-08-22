use super::binary::{Binary, OwnedBinary};
use crate::{Decoder, Encoder, Env, Error, NifResult, Term};

impl<'a> Decoder<'a> for String {
    #[inline]
    fn decode(term: Term<'a>) -> NifResult<Self> {
        let string: &str = Decoder::decode(term)?;
        Ok(string.to_string())
    }
}
impl<'a> Decoder<'a> for &'a str {
    #[inline]
    fn decode(term: Term<'a>) -> NifResult<Self> {
        let binary = Binary::from_term(term)?;
        match ::std::str::from_utf8(binary.as_slice()) {
            Ok(string) => Ok(string),
            Err(_) => Err(Error::BadArg),
        }
    }
}

impl Encoder for &str {
    fn encode<'b>(&self, env: Env<'b>) -> Term<'b> {
        (*self).encode(env)
    }
}

impl Encoder for str {
    fn encode<'b>(&self, env: Env<'b>) -> Term<'b> {
        let str_len = self.len();
        let mut bin = OwnedBinary::new(str_len).unwrap_or_else(|| {
            panic!("string encode failed: enif_alloc_binary({str_len}) failed (out of memory)")
        });
        bin.as_mut_slice().copy_from_slice(self.as_bytes());
        bin.release(env).to_term(env)
    }
}

impl Encoder for String {
    fn encode<'b>(&self, env: Env<'b>) -> Term<'b> {
        self.as_str().encode(env)
    }
}
