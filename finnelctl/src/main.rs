use anyhow::Result;

mod utils;

mod account;
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

    match config.command() {
        Commands::Account { .. } => account::run(&config)?,
        Commands::Record { .. } => record::run(&config)?,
        Commands::Category { .. } => category::run(&config)?,
        Commands::Merchant { .. } => merchant::run(&config)?,
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

    Ok(())
}
