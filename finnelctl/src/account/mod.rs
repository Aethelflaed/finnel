use anyhow::Result;

use crate::cli::{AccountCommands, Commands};
use crate::config::Config;

pub fn run(config: &Config) -> Result<()> {
    let Commands::Account { command } = config.command().clone().unwrap();

    match command {
        AccountCommands::List {} => list(command, &config),
        AccountCommands::Default { .. } => default(command, &config),
    }
}

fn list(_command: AccountCommands, config: &Config) -> Result<()> {
    Ok(())
}

fn default(command: AccountCommands, config: &Config) -> Result<()> {
    let AccountCommands::Default {
        account_name,
        reset,
    } = command
    else {
        anyhow::bail!("wrong command passed: {:?}", command);
    };

    if let Some(name) = account_name {
        Ok(config.database().set("default_account", name)?)
    } else if reset {
        Ok(config.database().reset("default_account")?)
    } else {
        let account_name = config
            .database()
            .get("default_account")?
            .unwrap_or("<not set>".to_string());
        println!("{}", account_name);
        Ok(())
    }
}
