use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

use crate::{Decoder, Encoder, Env, Error, NifResult, Term};

// IpAddr

impl Encoder for IpAddr {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        self.to_string().encode(env)
    }
}

impl<'a> Decoder<'a> for IpAddr {
    fn decode(term: Term<'a>) -> NifResult<Self> {
        let s: String = term.decode()?;
        s.parse().map_err(|_| Error::BadArg)
    }
}

// Ipv4Addr

impl Encoder for Ipv4Addr {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        self.to_string().encode(env)
    }
}

impl<'a> Decoder<'a> for Ipv4Addr {
    fn decode(term: Term<'a>) -> NifResult<Self> {
        let s: String = term.decode()?;
        s.parse().map_err(|_| Error::BadArg)
    }
}

// Ipv6Addr

impl Encoder for Ipv6Addr {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        self.to_string().encode(env)
    }
}

impl<'a> Decoder<'a> for Ipv6Addr {
    fn decode(term: Term<'a>) -> NifResult<Self> {
        let s: String = term.decode()?;
        s.parse().map_err(|_| Error::BadArg)
    }
}

// SocketAddr

impl Encoder for SocketAddr {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        (self.ip().to_string(), self.port() as i64).encode(env)
    }
}

impl<'a> Decoder<'a> for SocketAddr {
    fn decode(term: Term<'a>) -> NifResult<Self> {
        let (ip_str, port): (String, i64) = term.decode()?;
        let ip: IpAddr = ip_str.parse().map_err(|_| Error::BadArg)?;
        let port = u16::try_from(port).map_err(|_| Error::BadArg)?;
        Ok(SocketAddr::new(ip, port))
    }
}

// SocketAddrV4

impl Encoder for SocketAddrV4 {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        (self.ip().to_string(), self.port() as i64).encode(env)
    }
}

impl<'a> Decoder<'a> for SocketAddrV4 {
    fn decode(term: Term<'a>) -> NifResult<Self> {
        let (ip_str, port): (String, i64) = term.decode()?;
        let ip: Ipv4Addr = ip_str.parse().map_err(|_| Error::BadArg)?;
        let port = u16::try_from(port).map_err(|_| Error::BadArg)?;
        Ok(SocketAddrV4::new(ip, port))
    }
}

// SocketAddrV6

impl Encoder for SocketAddrV6 {
    fn encode<'a>(&self, env: Env<'a>) -> Term<'a> {
        (self.ip().to_string(), self.port() as i64).encode(env)
    }
}

impl<'a> Decoder<'a> for SocketAddrV6 {
    fn decode(term: Term<'a>) -> NifResult<Self> {
        let (ip_str, port): (String, i64) = term.decode()?;
        let ip: Ipv6Addr = ip_str.parse().map_err(|_| Error::BadArg)?;
        let port = u16::try_from(port).map_err(|_| Error::BadArg)?;
        Ok(SocketAddrV6::new(ip, port, 0, 0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

    #[test]
    fn trait_impls_exist() {
        fn assert_enc<T: Encoder>() {}
        fn assert_dec<'a, T: Decoder<'a>>() {}

        assert_enc::<IpAddr>();
        assert_dec::<IpAddr>();
        assert_enc::<Ipv4Addr>();
        assert_dec::<Ipv4Addr>();
        assert_enc::<Ipv6Addr>();
        assert_dec::<Ipv6Addr>();
        assert_enc::<SocketAddr>();
        assert_dec::<SocketAddr>();
        assert_enc::<SocketAddrV4>();
        assert_dec::<SocketAddrV4>();
        assert_enc::<SocketAddrV6>();
        assert_dec::<SocketAddrV6>();
    }

    #[test]
    fn socket_addr_parse_logic() {
        let raw = "192.168.1.1:443";
        let parsed: SocketAddr = raw.parse().unwrap();
        assert_eq!(parsed.ip().to_string(), "192.168.1.1");
        assert_eq!(parsed.port(), 443);

        let v6_raw = "[::1]:8080";
        let v6_parsed: SocketAddr = v6_raw.parse().unwrap();
        assert!(matches!(v6_parsed.ip(), IpAddr::V6(_)));
        assert_eq!(v6_parsed.port(), 8080);
    }
}
