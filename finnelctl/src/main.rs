use anyhow::Result;

mod utils;

mod account;
mod category;
mod cli;
mod config;
mod merchant;
mod record;

#[cfg(test)]
mod test;

use cli::Commands;
use config::Config;

fn main() -> Result<()> {
    let config = Config::try_parse()?;

    if let Some(command) = config.command() {
        match command {
            Commands::Account { .. } => account::run(&config)?,
            Commands::Record { .. } => record::run(&config)?,
            Commands::Category { .. } => category::run(&config)?,
            Commands::Merchant { .. } => merchant::run(&config)?,
            Commands::Reset { confirm } => {
                if *confirm {
                    std::fs::remove_file(config.database_path())?;
                }
            }
        }
    }

    Ok(())
}
