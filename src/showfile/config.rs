use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

#[derive(Debug, Clone, PartialEq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Config {
    address: SocketAddr,
}

impl Config {
    pub fn address(&self) -> SocketAddr {
        self.address
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            address: SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, crate::DEFAULT_PORT)),
        }
    }
}
