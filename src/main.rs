mod sys;

pub fn main() -> anyhow::Result<()> {
    sys::wsa_startup()?;
    let _sock = sys::create_raw_icmp_socket()?;
    Ok(())
}