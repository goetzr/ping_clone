use std::net::Ipv4Addr;

use clap::{Parser, Subcommand};

mod cli;
mod sys;
mod ping;

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
    let cli = Cli::parse();
    //sys::set_console_ctrl_handler(cli::console_ctrl_handler)?;
    
    // let hostname = "www.google.com";
    // let ttl = 100;
    // let timeout = 4;
    // let ip_addr = sys::resolve_hostname(hostname)?;
    // let reply = sys::send_ping(ip_addr, ttl, timeout)?;
    // println!("Reply received in {} milliseconds.", reply.RoundTripTime);
    Ok(())
}