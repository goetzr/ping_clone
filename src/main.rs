mod sys;

pub fn main() -> anyhow::Result<()> {
    sys::wsa_startup()?;
    Ok(())
}