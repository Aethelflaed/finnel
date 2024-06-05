use anyhow::Result;

use finnel::Database;

mod cli;
mod config;

pub fn run() -> Result<()> {
    let config = config::Config::try_parse()?;
    let db = Database::open(config.data_dir.join("db.finnel"))?;

    Ok(())
}
