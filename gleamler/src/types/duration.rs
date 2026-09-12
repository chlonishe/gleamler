use crate::{Decoder, Encoder, Env, Error, NifResult, Term};
use std::time::Duration;

/// Encodes `Duration` as microseconds (u128). Decodes from any integer term.
impl Encoder for Duration {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        self.as_micros().encode(env)
    }
}

impl<'a> Decoder<'a> for Duration {
    fn decode(term: Term<'a>) -> NifResult<Self> {
        let micros: u64 = term.decode()?;
        Ok(Duration::from_micros(micros))
    }
}

/// Erlang-style `{MegaSecs, Secs, MicroSecs}` tuple
pub struct ErlangTimestamp(pub Duration);

impl Encoder for ErlangTimestamp {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        let secs = self.0.as_secs();
        let mega = secs / 1_000_000;
        let rem_secs = secs % 1_000_000;
        let micro = self.0.subsec_micros() as u64;
        (mega, rem_secs, micro).encode(env)
    }
}

impl<'a> Decoder<'a> for ErlangTimestamp {
    fn decode(term: Term<'a>) -> NifResult<Self> {
        let (mega, secs, micro): (u64, u64, u64) = term.decode()?;
        let total_secs = mega
            .checked_mul(1_000_000)
            .and_then(|s| s.checked_add(secs))
            .ok_or(Error::BadArg)?;
        let dur = Duration::from_secs(total_secs)
            .checked_add(Duration::from_micros(micro))
            .ok_or(Error::BadArg)?;
        Ok(ErlangTimestamp(dur))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_impls_exist() {
        fn assert_encoder<T: Encoder>() {}
        fn assert_decoder<'a, T: Decoder<'a>>() {}
        assert_encoder::<Duration>();
        assert_decoder::<Duration>();
        assert_encoder::<ErlangTimestamp>();
        assert_decoder::<ErlangTimestamp>();
    }

    #[test]
    fn erlang_timestamp_decomposition() {
        let ts = ErlangTimestamp(Duration::from_micros(1_000_001));
        assert_eq!(ts.0.as_micros(), 1_000_001);
    }
}
