use crate::{Decoder, Encoder, Env, Error, NifResult, Term};

const EXTERNAL_TERM_FORMAT_VERSION: u8 = 131;
const SMALL_BIG_EXT: u8 = 110;

impl Encoder for i128 {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        if let Ok(int) = i64::try_from(*self) {
            int.encode(env)
        } else {
            let mut etf = [0u8; 4 + 16];
            etf[0] = EXTERNAL_TERM_FORMAT_VERSION;
            etf[1] = SMALL_BIG_EXT;
            etf[2] = 16;
            if *self < 0 {
                etf[3] = 1;
                let bytes = self.unsigned_abs().to_le_bytes();
                etf[4..].copy_from_slice(&bytes);
            } else {
                etf[3] = 0;
                let bytes = (*self as u128).to_le_bytes();
                etf[4..].copy_from_slice(&bytes);
            }
            match env.binary_to_term(&etf) {
                Some((term, _)) => term,
                None => panic!("i128 encode failed: enif_binary_to_term failed (out of memory)"),
            }
        }
    }
}

impl Encoder for u128 {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        if let Ok(int) = u64::try_from(*self) {
            int.encode(env)
        } else {
            let mut etf = [0u8; 4 + 16];
            etf[0] = EXTERNAL_TERM_FORMAT_VERSION;
            etf[1] = SMALL_BIG_EXT;
            etf[2] = 16;
            etf[4..].copy_from_slice(&self.to_le_bytes());
            match env.binary_to_term(&etf) {
                Some((term, _)) => term,
                None => panic!("u128 encode failed: enif_binary_to_term failed (out of memory)"),
            }
        }
    }
}

impl<'a> Decoder<'a> for i128 {
    fn decode(term: Term<'a>) -> NifResult<Self> {
        if !term.is_integer() {
            return Err(Error::BadArg);
        }

        if let Ok(int) = term.decode::<i64>() {
            return Ok(int as i128);
        }

        let input = term.to_binary();
        let input = input.as_slice();
        if input.len() < 4 {
            return Err(Error::BadArg);
        }

        if input[0] != EXTERNAL_TERM_FORMAT_VERSION {
            return Err(Error::BadArg);
        }

        if input[1] != SMALL_BIG_EXT {
            return Err(Error::BadArg);
        }

        let n = input[2] as usize;
        if n > 16 {
            return Err(Error::BadArg);
        }

        if input.len() < 4 + n {
            return Err(Error::BadArg);
        }

        let is_pos = input[3] == 0;

        let mut res = [0u8; 16];
        res[..n].copy_from_slice(&input[4..4 + n]);

        let raw = u128::from_le_bytes(res);
        
        if is_pos {
            if raw > i128::MAX as u128 {
                return Err(Error::BadArg);
            }
            Ok(raw as i128)
        } else {
            let min_magnitude = (i128::MAX as u128).wrapping_add(1);
            if raw > min_magnitude {
                return Err(Error::BadArg);
            }
            Ok((raw as i128).wrapping_neg())
        }
    }
}

impl<'a> Decoder<'a> for u128 {
    fn decode(term: Term<'a>) -> NifResult<Self> {
        if !term.is_integer() {
            return Err(Error::BadArg);
        }

        if let Ok(int) = term.decode::<u64>() {
            return Ok(int as u128);
        }

        let input = term.to_binary();
        let input = input.as_slice();

        if input.len() < 4 {
            return Err(Error::BadArg);
        }

        if input[0] != EXTERNAL_TERM_FORMAT_VERSION {
            return Err(Error::BadArg);
        }

        if input[1] != SMALL_BIG_EXT {
            return Err(Error::BadArg);
        }

        let n = input[2] as usize;
        if n > 16 {
            return Err(Error::BadArg);
        }

        if input.len() < 4 + n {
            return Err(Error::BadArg);
        }

        if input[3] != 0 {
            return Err(Error::BadArg);
        }

        let mut res = [0u8; 16];
        res[..n].copy_from_slice(&input[4..4 + n]);
        Ok(u128::from_le_bytes(res))
    }
}