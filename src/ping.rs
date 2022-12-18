use std::net::Ipv4Addr;
use std::fmt;

use windows::Win32::NetworkManagement::IpHelper::*;

use crate::sys::{icmp_create};
use crate::Cli;

#[derive(Debug)]
pub enum Error {
    Create(Box<dyn std::error::Error>),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Create(e) => write!(f, "failed to create the ping manager: {e}"),
        }
    }
}

impl std::error::Error for Error {}

type Result<T> = std::result::Result<T, Error>;

struct PingManager {
    cli: Cli,
    stats: PingStats,
    icmp_handle: IcmpHandle,
}

impl PingManager {
    pub fn new(cli: Cli) -> Result<Self> {
        let icmp_handle = icmp_create().map_err(|e| Error::Create(Box::new(e)))?;
        Ok(PingManager { cli, icmp_handle, stats: PingStats::new() })
    }
}

struct PingStats {
    requests_sent: u32,
    replies_rcvd: u32
}

impl PingStats {
    fn new() -> Self {
        PingStats { requests_sent: 0, replies_rcvd: 0 }
    }
}