use anyhow::Result;

#[macro_use]
mod utils;

mod account;
mod calendar;
mod category;
mod cli;
mod config;
mod merchant;
mod record;

#[cfg(test)]
pub mod test;

use cli::Commands;
use config::Config;

fn main() -> Result<()> {
    let config = Config::try_parse()?;

    if let Some(command) = config.command() {
        match command {
            Commands::Account(cmd) => account::run(&config, cmd)?,
            Commands::Record(cmd) => record::run(&config, cmd)?,
            Commands::Category(cmd) => category::run(&config, cmd)?,
            Commands::Merchant(cmd) => merchant::run(&config, cmd)?,
            Commands::Calendar(cmd) => calendar::run(&config, cmd)?,
            Commands::Consolidate { .. } => {
                let conn = &mut config.database()?;

                finnel::merchant::consolidate(conn)?;
                finnel::category::consolidate(conn)?;
                finnel::record::consolidate(conn)?;
            }
            Commands::Reset { confirm } => {
                if *confirm && utils::confirm()? {
                    std::fs::remove_file(config.database_path())?;
                } else {
                    anyhow::bail!("operation requires confirmation");
                }
            }
        }
    } else {
        anyhow::bail!("No command provided");
    }

    Ok(())
}
