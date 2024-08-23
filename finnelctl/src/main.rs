use anyhow::Result;

#[macro_use]
mod utils;

mod account;
mod calendar;
mod category;
mod cli;
mod config;
mod import;
mod merchant;
mod record;
mod report;

#[cfg(test)]
pub mod test;

use cli::Commands;
use config::Config;

fn main() -> Result<()> {
    let config = Config::try_parse()?;

    setup_log(config.log_level_filter())?;

    if let Some(command) = config.command() {
        log::debug!("Executing {:?}", command);
        match command {
            Commands::Account(cmd) => account::run(&config, cmd)?,
            Commands::Record(cmd) => record::run(&config, cmd)?,
            Commands::Category(cmd) => category::run(&config, cmd)?,
            Commands::Merchant(cmd) => merchant::run(&config, cmd)?,
            Commands::Calendar(cmd) => calendar::run(&config, cmd)?,
            Commands::Report(cmd) => report::run(&config, cmd)?,
            Commands::Import(cmd) => import::run(&config, cmd)?,
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

fn setup_log(level: log::LevelFilter) -> Result<()> {
    use env_logger::{Builder, Env};
    use systemd_journal_logger::{connected_to_journal, JournalLog};

    // If the output streams of this process are directly connected to the
    // systemd journal log directly to the journal to preserve structured
    // log entries (e.g. proper multiline messages, metadata fields, etc.)
    if connected_to_journal() {
        JournalLog::new()
            .unwrap()
            .with_extra_fields(vec![("VERSION", env!("CARGO_PKG_VERSION"))])
            .install()?;
    } else {
        let name = String::from(env!("CARGO_PKG_NAME"))
            .replace('-', "_")
            .to_uppercase();
        let env = Env::new()
            .filter(format!("{}_LOG", name))
            .write_style(format!("{}_LOG_STYLE", name));

        Builder::new()
            .filter_level(log::LevelFilter::Trace)
            .parse_env(env)
            .try_init()?;
    }

    log::set_max_level(level);

    Ok(())
}
