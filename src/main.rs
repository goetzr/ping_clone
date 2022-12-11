mod sys;

pub fn main() -> anyhow::Result<()> {
    let hostname = "www.google.com";
    let ip_addr = sys::resolve_hostname(hostname)?;
    println!("{} resolved to {}", hostname, ip_addr);
    Ok(())
}