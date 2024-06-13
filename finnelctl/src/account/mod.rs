use anyhow::Result;

use crate::cli::{AccountCommands, Commands};
use crate::config::Config;

use finnel::account::Account;
use finnel::{Amount, Database, Entity, Error};

pub fn run(config: &Config) -> Result<()> {
    let Commands::Account { command } = config.command().clone().unwrap();

    match command {
        AccountCommands::List {} => list(command, &config),
        AccountCommands::Create { .. } => create(command, &config),
        AccountCommands::Show { .. } => show(command, &config),
        AccountCommands::Delete { .. } => delete(command, &config),
        AccountCommands::Default { .. } => command_default(command, &config),
    }
}

pub fn by_name_or_default(db: &Database, name: Option<String>) -> Result<Account> {
    if let Some(account_name) = name {
        Ok(Account::find_by_name(&db, &account_name)?)
    } else {
        match default(&db) {
            Ok(None) => Err(Error::NotFound.into()),
            Ok(Some(account)) => Ok(account),
            Err(e) => Err(e)
        }
    }
}

pub fn default(db: &Database) -> Result<Option<Account>> {
    if let Some(account_name) = db.get("default_account")? {
        match Account::find_by_name(&db, &account_name) {
            Ok(entity) => Ok(Some(entity)),
            Err(Error::NotFound) => {
                db.reset("default_account")?;
                Ok(None)
            }
            Err(error) => Err(error.into()),
        }
    } else {
        Ok(None)
    }
}

fn list(_command: AccountCommands, config: &Config) -> Result<()> {
    Account::for_each(&config.database(), |account| {
        println!("{}", account.name());
    })?;

    Ok(())
}

fn create(command: AccountCommands, config: &Config) -> Result<()> {
    let AccountCommands::Create { account_name } = command else {
        anyhow::bail!("wrong command passed: {:?}", command);
    };

    let mut account = Account::new(account_name);
    account.save(&config.database())?;
    Ok(())
}

fn show(command: AccountCommands, config: &Config) -> Result<()> {
    let AccountCommands::Show { account_name } = command else {
        anyhow::bail!("wrong command passed: {:?}", command);
    };

    let db = config.database();
    let mut account = by_name_or_default(&db, account_name)?;

    let Amount(amount, currency) = account.balance();
    println!("{} {}", currency.code(), amount);
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

    let mut account = by_name_or_default(&db, account_name)?;

    if confirm {
        account.delete(&mut db)?;
    } else {
        anyhow::bail!("operation requires confirmation flag");
    }
    Ok(())
}

fn command_default(command: AccountCommands, config: &Config) -> Result<()> {
    let AccountCommands::Default {
        account_name,
        reset,
    } = command
    else {
        anyhow::bail!("wrong command passed: {:?}", command);
    };

    let db = config.database();

    if let Some(name) = account_name {
        let account = Account::find_by_name(&db, &name)?;
        Ok(db.set("default_account", account.name())?)
    } else if reset {
        Ok(db.reset("default_account")?)
    } else {
        let account_name = default(&db)?
            .map(|a| a.name().to_string())
            .unwrap_or("<not set>".to_string());
        println!("{}", account_name);
        Ok(())
    }
}
