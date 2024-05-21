use std::{env, net::SocketAddrV4, path::PathBuf};

#[derive(Debug, Clone)]
pub struct Settings {
    pub leases_file: PathBuf,
    pub bind_addr: SocketAddrV4,
}

impl Settings {
    /// Creates default settings where each field may be overwritten by an
    /// environment variable of the same name, but capitalized.
    pub fn new() -> Self {
        let leases_file: PathBuf = env::var("LEASES_FILE")
            .map(|s| s.into())
            .unwrap_or("/var/db/kea/dhcp4.leases".into());

        // TODO: catch the parse on envvar parse
        let bind_addr: SocketAddrV4 = env::var("BIND_ADDR")
            .map(|s| s.parse::<SocketAddrV4>().unwrap())
            .unwrap_or("127.0.0.1:8080".parse().unwrap());

        Self {
            leases_file,
            bind_addr,
        }
    }
}
