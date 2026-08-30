use crate::{Decoder, Encoder, Env, NifResult, Term};
use std::time::Duration;

/// Encodes `Duration` as microseconds (u64). Decodes from any integer term
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
        let total_micros = self.0.as_micros() as u64;
        let mega = total_micros / 1_000_000_000_000u64;
        let secs = (total_micros % 1_000_000_000_000u64) / 1_000_000u64;
        let micro = total_micros % 1_000_000u64;
        (mega, secs, micro).encode(env)
    }
}

impl<'a> Decoder<'a> for ErlangTimestamp {
    fn decode(term: Term<'a>) -> NifResult<Self> {
        let (mega, secs, micro): (u64, u64, u64) = term.decode()?;
        let total = mega * 1_000_000_000_000u64 + secs * 1_000_000u64 + micro;
        Ok(ErlangTimestamp(Duration::from_micros(total)))
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
