use anyhow::Result;
use rummage::cli;

fn main() -> Result<()> {
    cli::parse()?;
    Ok(())
}
