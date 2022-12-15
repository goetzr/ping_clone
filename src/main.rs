use std::time::Duration;

mod sys;
mod cli;

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
    if !sys::set_console_ctrl_handler(cli::console_ctrl_handler) {
        println!("ERROR: failed to set the console control handler.");
        std::process::exit(1);
    }
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