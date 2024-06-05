use anyhow::Result;

mod cli;
mod config;

pub fn run() -> Result<()> {
    let config = config::Config::try_parse()?;

    Ok(())
}
