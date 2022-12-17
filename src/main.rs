use std::time::Duration;

mod cli;
mod sys;
mod ping;

pub fn main() -> anyhow::Result<()> {
    /*
    Options to implement:
        -t,
        -a,
        -n count,
        -l size,
        -f,
        -i TTL,
        -w timeout,
        -S srcaddr
     */
    sys::set_console_ctrl_handler(cli::console_ctrl_handler)?;
    println!("Pinging...");
    loop {
        std::thread::sleep(Duration::from_millis(100));
    }
    // let hostname = "www.google.com";
    // let ttl = 100;
    // let timeout = 4;
    // let ip_addr = sys::resolve_hostname(hostname)?;
    // let reply = sys::send_ping(ip_addr, ttl, timeout)?;
    // println!("Reply received in {} milliseconds.", reply.RoundTripTime);
    Ok(())
}