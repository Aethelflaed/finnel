use anyhow::Result;

mod account;
mod cli;
mod config;

#[cfg(test)]
mod test;

use cli::Commands;
use config::Config;

fn main() -> Result<()> {
    let config = Config::try_parse()?;

    if let Some(command) = config.command() {
        match command {
            Commands::Account { .. } => account::run(&config)?,
            Commands::Record { .. } => {
                todo!()
            }
        }
    }

    Ok(())
}
