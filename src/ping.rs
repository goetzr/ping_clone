use std::net::Ipv4Addr;
use std::fmt;

use windows::Win32::NetworkManagement::IpHelper::*;

use crate::sys::{icmp_create};

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
    dst_addr: Ipv4Addr,
    icmp_handle: IcmpHandle,
}

impl PingManager {
    pub fn new(dst_addr: Ipv4Addr) -> Result<Self> {
        let icmp_handle = icmp_create().map_err(|e| Error::Create(Box::new(e)))?;
        Ok(PingManager { dst_addr, icmp_handle })
    }
}

struct PingStats {
    requests_sent: u32,
    replies_rcvd: u32
}