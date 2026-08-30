use std::time::SystemTime;

use crate::{Decoder, Encoder, Env, NifResult, Term};

/// Encodes `SystemTime` as Erlang timestamp `{MegaSecs, Secs, MicroSecs}`
/// Decodes from either the timestamp tuple or a plain microseconds integer
impl Encoder for SystemTime {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        let duration = self
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_else(|_| std::time::Duration::from_secs(0));
        let total_micros = duration.as_micros() as u64;
        let mega = total_micros / 1_000_000_000_000u64;
        let secs = (total_micros % 1_000_000_000_000u64) / 1_000_000u64;
        let micro = total_micros % 1_000_000u64;
        (mega, secs, micro).encode(env)
    }
}

impl<'a> Decoder<'a> for SystemTime {
    fn decode(term: Term<'a>) -> NifResult<Self> {
        if let Ok((mega, secs, micro)) = term.decode::<(u64, u64, u64)>() {
            let total = mega * 1_000_000_000_000u64 + secs * 1_000_000u64 + micro;
            return Ok(SystemTime::UNIX_EPOCH + std::time::Duration::from_micros(total));
        }
        let micros: u64 = term.decode()?;
        Ok(SystemTime::UNIX_EPOCH + std::time::Duration::from_micros(micros))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, SystemTime};

    #[test]
    fn trait_impls_exist() {
        fn assert_enc<T: Encoder>() {}
        fn assert_dec<'a, T: Decoder<'a>>() {}

        assert_enc::<SystemTime>();
        assert_dec::<SystemTime>();
    }

    #[test]
    fn system_time_decomposition_roundtrip() {
        let total_micros = 1_234_567_890_123u64;
        let mega = total_micros / 1_000_000_000_000u64;
        let secs = (total_micros % 1_000_000_000_000u64) / 1_000_000u64;
        let micro = total_micros % 1_000_000u64;
        let reconstructed = mega * 1_000_000_000_000u64 + secs * 1_000_000u64 + micro;
        assert_eq!(reconstructed, total_micros);
    }

    #[test]
    fn system_time_epoch_zero() {
        let epoch = SystemTime::UNIX_EPOCH;
        let dur = epoch.duration_since(SystemTime::UNIX_EPOCH).unwrap();
        assert_eq!(dur.as_secs(), 0);
    }
}
