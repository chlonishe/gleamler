use super::binary::{Binary, NewBinary, OwnedBinary};
use crate::{Decoder, Encoder, Env, Error, NifResult, Term};
use crate::types::atom;

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

use std::io::Write;

impl Encoder for &str {
    fn encode<'b>(&self, env: Env<'b>) -> Term<'b> {
        (*self).encode(env)
    }
}

impl Encoder for str {
    fn encode<'b>(&self, env: Env<'b>) -> Term<'b> {
        let str_len = self.len();
        if str_len <= 64 {
            let mut bin = NewBinary::new(env, str_len);
            bin.as_mut_slice().copy_from_slice(self.as_bytes());
            bin.into()
        } else {
            let mut bin = match OwnedBinary::new(str_len) {
                Some(bin) => bin,
                None => panic!("string encode failed: enif_alloc_binary({str_len}) failed (out of memory)"),
            };
            bin.as_mut_slice()
                .write_all(self.as_bytes())
                .expect("memory copy of string failed");
            bin.release(env).to_term(env)
        }
    }
}

impl Encoder for String {
    fn encode<'b>(&self, env: Env<'b>) -> Term<'b> {
        self.as_str().encode(env)
    }
}
