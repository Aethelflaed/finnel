use anyhow::Result;
use std::borrow::Cow;

use crate::cli::{account::Command, Commands};
use crate::config::Config;

use finnel::account::Account;
use finnel::{Amount, Database, DatabaseTrait, Entity, Error};

use tabled::{Table, Tabled};

#[derive(derive_more::From)]
struct AccountToDisplay(Account);

impl Tabled for AccountToDisplay {
    const LENGTH: usize = 2;

    fn fields(&self) -> Vec<Cow<'_, str>> {
        vec![
            self.id(),
            self.0.name.clone().into(),
            self.0.balance().to_string().into(),
        ]
    }

    fn headers() -> Vec<Cow<'static, str>> {
        vec!["id".into(), "name".into(), "balance".into()]
    }
}

impl AccountToDisplay {
    fn id(&self) -> Cow<'_, str> {
        if let Some(id) = self.0.id() {
            id.value().to_string().into()
        } else {
            Default::default()
        }
    }
}

pub fn run(config: &Config) -> Result<()> {
    let Commands::Account(command) = config.command().clone().unwrap() else {
        anyhow::bail!("wrong command passed: {:?}", config.command());
    };

    match command {
        Command::List {} => list(command, config),
        Command::Create { .. } => create(command, config),
        Command::Show { .. } => show(command, config),
        Command::Delete { .. } => delete(command, config),
        Command::Default { .. } => command_default(command, config),
    }
}

pub fn default(db: &Database) -> Result<Option<Account>> {
    if let Some(account_name) = db.get("default_account")? {
        match Account::find_by_name(db, &account_name) {
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

fn list(_command: Command, config: &Config) -> Result<()> {
    let db = &config.database()?;

    let mut accounts = Vec::<AccountToDisplay>::new();

    Account::for_each(db, |account| {
        accounts.push(account.into());
    })?;

    println!("{}", Table::new(accounts));

    Ok(())
}

fn create(command: Command, config: &Config) -> Result<()> {
    let Command::Create { name } = command else {
        anyhow::bail!("wrong command passed: {:?}", command);
    };

    let db = &config.database()?;

    let mut account = Account::new(name);
    account.save(db)?;
    Ok(())
}

fn show(command: Command, config: &Config) -> Result<()> {
    let Command::Show { .. } = command else {
        anyhow::bail!("wrong command passed: {:?}", command);
    };

    let db = &config.database()?;
    let account = config.account_or_default(db)?;

    println!("{} | {}", account.id().unwrap().value(), account.name);
    println!("\tBalance: {}", account.balance());

    Ok(())
}

fn delete(command: Command, config: &Config) -> Result<()> {
    let Command::Delete { confirm } = command else {
        anyhow::bail!("wrong command passed: {:?}", command);
    };

    let mut db = config.database()?;

    let mut account = config.account_or_default(&db)?;

    if confirm {
        account.delete(&mut db)?;
    } else {
        anyhow::bail!("operation requires confirmation flag");
    }
    Ok(())
}

fn command_default(command: Command, config: &Config) -> Result<()> {
    let Command::Default { reset } = command else {
        anyhow::bail!("wrong command passed: {:?}", command);
    };

    let db = config.database()?;

    if let Some(name) = config.account_name() {
        let account = Account::find_by_name(&db, name)?;
        Ok(db.set("default_account", account.name)?)
    } else if reset {
        Ok(db.reset("default_account")?)
    } else {
        let account_name = default(&db)?
            .map(|a| a.name.clone())
            .unwrap_or("<not set>".to_string());
        println!("{}", account_name);
        Ok(())
    }
}
