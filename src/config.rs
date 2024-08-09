use std::{env, net::SocketAddrV4};

#[derive(Debug, Clone)]
pub struct Settings {
    pub leases_db: String,
    pub bind_addr: SocketAddrV4,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Could not parse binding address `{0}`")]
    AddrParse(String),
}

impl Settings {
    /// Creates default settings where each field may be overwritten by an
    /// environment variable of the same name, but capitalized.
    pub fn new() -> Result<Self, Error> {
        let leases_db = env::var("LEASES_DB")
            .unwrap_or("/var/lib/kea/kea-leases4.csv*".into());

        let bind_addr = match env::var("BIND_ADDR") {
            Ok(addr) => addr
                .parse::<SocketAddrV4>()
                .map_err(|_| Error::AddrParse(addr.clone()))?,
            Err(_) => "127.0.0.1:8080".parse::<SocketAddrV4>().unwrap(),
        };

        Ok(Self {
            leases_db,
            bind_addr,
        })
    }
}
