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
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};

    #[test]
    fn ipv4_roundtrip_string() {
        let ip = Ipv4Addr::new(192, 168, 1, 1);
        assert_eq!(ip.to_string().parse::<Ipv4Addr>().unwrap(), ip);
    }

    #[test]
    fn ipv6_roundtrip_string() {
        let ip = Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 1);
        assert_eq!(ip.to_string().parse::<Ipv6Addr>().unwrap(), ip);
    }

    #[test]
    fn socket_addr_v4_encode_logic() {
        let addr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 8080);
        let (ip, port) = (addr.ip().to_string(), addr.port() as i64);
        assert_eq!(ip, "127.0.0.1");
        assert_eq!(port, 8080);
    }

    #[test]
    fn socket_addr_v6_encode_logic() {
        let addr = SocketAddrV6::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), 443, 0, 0);
        let (ip, port) = (addr.ip().to_string(), addr.port() as i64);
        assert_eq!(ip, "::1");
        assert_eq!(port, 443);
    }

    #[test]
    fn ip_addr_from_str_rejects_garbage() {
        assert!("not-an-ip".parse::<IpAddr>().is_err());
        assert!("999.999.999.999".parse::<IpAddr>().is_err());
        assert!(":::".parse::<IpAddr>().is_err());
    }

    #[test]
    fn socket_addr_tuple_decoder_logic() {
        let ip: IpAddr = "127.0.0.1".parse().unwrap();
        let port: u16 = 8080;
        let reconstructed = SocketAddr::new(ip, port);
        assert_eq!(reconstructed.to_string(), "127.0.0.1:8080");
    }
}
