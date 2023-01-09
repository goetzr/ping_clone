use std::net::Ipv4Addr;
use std::fmt;

use windows::Win32::NetworkManagement::IpHelper::*;

use crate::sys::{icmp_create};
use crate::Cli;

#[derive(Debug)]
pub enum Error {
    Create(Box<dyn std::error::Error + 'static + Send + Sync>),
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

pub struct PingManager {
    cli: Cli,
    stats: PingStats,
    icmp_handle: IcmpHandle,
}

impl PingManager {
    pub fn new(cli: Cli) -> Result<Self> {
        let icmp_handle = unsafe { IcmpCreateFile().map_err(|e| {
            Error
        }) }
        //let icmp_handle = icmp_create().map_err(|e| Error::Create(Box::new(e)))?;
        // unsafe {
        //     IcmpCreateFile().map_err(|e| {
        //         Error::OpenIcmpHandle(Win32Error::new(e.code().0 as u32, e.message().to_string()))
        //     })
        // }

        // TODO: ping statistics need to be global so the console control handler can use them.
        Ok(PingManager { cli, icmp_handle, stats: PingStats::new() })
    }

    pub fn start_pinging(&self) {
        for _ in 0.. {
            println!("Sending ping...");
            std::thread::sleep(std::time::Duration::from_secs(1));
            if *crate::STOP_CHANNEL.lock().unwrap() {
                break;
            }
            let mut display = crate::DISPLAY_STATS.lock().unwrap();
            if *display {
                *display = false;
                println!("Displaying stats");
            } 
        }
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