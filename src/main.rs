use std::net::Ipv4Addr;
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Mutex;
use std::thread;

use clap::Parser;
use lazy_static::lazy_static;
use windows::Win32::Foundation::*;
use windows::Win32::System::Console::*;

mod ping;
mod sys;

use ping::PingManager;

lazy_static! {
    pub static ref STOP_CHANNEL: Mutex<bool> = Mutex::new(false);
    pub static ref DISPLAY_STATS: Mutex<bool> = Mutex::new(false);
}

#[derive(Parser)]
pub struct Cli {
    /// Ping the specified host until stopped.
    /// To see statistic and continue - type Control-Break;
    /// To stop - type Control-C.
    #[arg(short = 't', verbatim_doc_comment)]
    until_stopped: bool,
    /// Resolve addresses to hostnames.
    #[arg(short = 'a', verbatim_doc_comment)]
    resolve_addresses: bool,
    /// Number of echo requests to send.
    #[arg(short = 'n', default_value_t = 4, verbatim_doc_comment)]
    count: u32,
    /// Send buffer size.
    #[arg(short = 'l', default_value_t = 32, verbatim_doc_comment)]
    size: u32,
    /// Set Don't Fragment flag in packet.
    #[arg(short = 'f', verbatim_doc_comment)]
    dont_fragment: bool,
    /// Time To Live.
    #[arg(short = 'i', verbatim_doc_comment)]
    ttl: Option<u32>,
    /// Timeout in milliseconds to wait for each reply.
    #[arg(short = 'w', default_value_t = 4000, verbatim_doc_comment)]
    timeout: u32,
    /// Source address to use.
    #[arg(short = 'S', verbatim_doc_comment)]
    srcaddr: Option<Ipv4Addr>,
}

pub fn main() -> anyhow::Result<()> {
    if !unsafe { SetConsoleCtrlHandler(Some(console_ctrl_handler)) }.as_bool() {
        Err(...)
    }
    sys::set_console_ctrl_handler(console_ctrl_handler)?;

    let cli = Cli::parse();
    let mgr = PingManager::new(cli)?;
    mgr.start_pinging();

    // let hostname = "www.google.com";
    // let ttl = 100;
    // let timeout = 4;
    // let ip_addr = sys::resolve_hostname(hostname)?;
    // let reply = sys::send_ping(ip_addr, ttl, timeout)?;
    // println!("Reply received in {} milliseconds.", reply.RoundTripTime);
    Ok(())
}

unsafe extern "system" fn console_ctrl_handler(ctrl_type: u32) -> BOOL {
    //println!("Control key {} pressed", ctrl_type);
    // TODO: Print ping statistics here, not in handlers.
    match ctrl_type {
        CTRL_C_EVENT => {
            //println!("CTRL-C pressed");
            let mut flag = STOP_CHANNEL.lock().unwrap();
            *flag = true;
            // Intentionally return false so the application is terminated.
            false.into()
        }
        CTRL_BREAK_EVENT => {
            // Fn+Ctrl+P on Lenovo laptop.
            //println!("CTRL-BREAK pressed");
            let mut flag = DISPLAY_STATS.lock().unwrap();
            *flag = true;
            true.into()
        }
        _ => false.into(),
    }
}

fn ping(cli: Cli) {
    let mut count = cli.count;
    while cli.until_stopped || count > 0 {
        // Send ping
    }
}
