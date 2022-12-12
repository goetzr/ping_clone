mod sys;

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
    let hostname = "www.google.com";
    let ttl = 100;
    let timeout = 4;
    let ip_addr = sys::resolve_hostname(hostname)?;
    let reply = sys::send_ping(ip_addr, ttl, timeout)?;
    println!("Reply received in {} milliseconds.", reply.RoundTripTime);
    Ok(())
}