use anyhow::Result;

use crate::cli::{AccountCommands, Commands};
use crate::config::Config;

use finnel::database::Entity;
use finnel::account::Account;

pub fn run(config: &Config) -> Result<()> {
    let Commands::Account { command } = config.command().clone().unwrap();

    match command {
        AccountCommands::List {} => list(command, &config),
        AccountCommands::Create { .. } => create(command, &config),
        AccountCommands::Delete { .. } => delete(command, &config),
        AccountCommands::Default { .. } => default(command, &config),
    }
}

fn list(_command: AccountCommands, config: &Config) -> Result<()> {
    Account::for_each(&config.database(), |account| {
        println!("{}", account.name());
    })?;

    Ok(())
}

fn create(command: AccountCommands, config: &Config) -> Result<()> {
    let AccountCommands::Create {
        account_name,
    } = command
    else {
        anyhow::bail!("wrong command passed: {:?}", command);
    };

    let mut account = Account::new(account_name);
    account.save(&config.database())?;
    Ok(())
}

fn delete(command: AccountCommands, config: &Config) -> Result<()> {
    let AccountCommands::Delete {
        account_name,
        confirm,
    } = command
    else {
        anyhow::bail!("wrong command passed: {:?}", command);
    };

    let mut db = config.database();

    let mut account = Account::find_by_name(&db, &account_name)?;

    if confirm {
        account.delete(&mut db)?;
    } else {
        anyhow::bail!("operation requires confirmation flag");
    }
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
