use std::time::{Duration, SystemTime};

use crate::{Decoder, Encoder, Env, Error, NifResult, Term};

impl Encoder for SystemTime {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        let duration = self
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_else(|_| Duration::from_secs(0));
        let secs = duration.as_secs();
        let mega = secs / 1_000_000;
        let rem_secs = secs % 1_000_000;
        let micro = duration.subsec_micros() as u64;
        (mega, rem_secs, micro).encode(env)
    }
}

impl<'a> Decoder<'a> for SystemTime {
    fn decode(term: Term<'a>) -> NifResult<Self> {
        if let Ok((mega, secs, micro)) = term.decode::<(u64, u64, u64)>() {
            let total_secs = mega
                .checked_mul(1_000_000)
                .and_then(|s| s.checked_add(secs))
                .ok_or(Error::BadArg)?;
            let dur = Duration::from_secs(total_secs)
                .checked_add(Duration::from_micros(micro))
                .ok_or(Error::BadArg)?;
            return Ok(SystemTime::UNIX_EPOCH + dur);
        }
        let micros: u64 = term.decode()?;
        Ok(SystemTime::UNIX_EPOCH + Duration::from_micros(micros))
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, SystemTime};

    #[test]
    fn system_time_epoch_zero() {
        let epoch = SystemTime::UNIX_EPOCH;
        let dur = epoch.duration_since(SystemTime::UNIX_EPOCH).unwrap();
        assert_eq!(dur.as_secs(), 0);
    }

    #[test]
    fn system_time_future() {
        let future = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000_000_000);
        let total_micros = future
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_micros() as u64;
        assert_eq!(total_micros, 1_000_000_000_000_000u64);
    }

    #[test]
    fn system_time_decomposition_identity() {
        let total = 12_345_678_901_234u64;
        let mega = total / 1_000_000_000_000u64;
        let secs = (total % 1_000_000_000_000u64) / 1_000_000u64;
        let micro = total % 1_000_000u64;
        assert_eq!(mega, 12);
        assert_eq!(secs, 345_678);
        assert_eq!(micro, 901_234);
        let reconstructed = mega * 1_000_000_000_000u64 + secs * 1_000_000u64 + micro;
        assert_eq!(reconstructed, total);
    }

    #[test]
    fn system_time_before_epoch_clamped_in_encoder() {
        let before = SystemTime::UNIX_EPOCH - Duration::from_micros(1);
        let dur = before.duration_since(SystemTime::UNIX_EPOCH);
        assert!(dur.is_err());
    }

    #[test]
    fn system_time_decoder_from_tuple() {
        let total = 1_000_000u64 + 500_000u64;
        assert_eq!(total, 1_500_000u64);
    }

    #[test]
    fn system_time_decoder_from_integer() {
        let micros = 42_000_000u64;
        let t = SystemTime::UNIX_EPOCH + Duration::from_micros(micros);
        let since_epoch = t.duration_since(SystemTime::UNIX_EPOCH).unwrap();
        assert_eq!(since_epoch.as_secs(), 42);
    }
}
