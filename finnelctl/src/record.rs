use anyhow::Result;

use crate::cli::{Commands, RecordCommands};
use crate::config::Config;

use finnel::{
    record::{NewRecord, QueryRecord},
    Account, Database, Entity, Query,
};

struct RecordCmd<'a> {
    _config: &'a Config,
    db: &'a Database,
    account: Account,
    command: RecordCommands,
}

pub fn run(config: &Config) -> Result<()> {
    let Commands::Record { command } = config.command().clone().unwrap() else {
        anyhow::bail!("wrong command passed: {:?}", config.command());
    };

    let db = &config.database()?;
    let mut cmd = RecordCmd {
        account: config.account_or_default(db)?,
        db,
        _config: config,
        command: command.clone(),
    };

    match command {
        RecordCommands::Add { .. } => cmd.add(),
        RecordCommands::List { .. } => cmd.list(),
    }
}

impl RecordCmd<'_> {
    fn add(&mut self) -> Result<()> {
        let RecordCommands::Add {
            amount,
            description,
            direction,
            mode,
            ..
        } = &self.command
        else {
            anyhow::bail!("wrong command passed: {:?}", self.command);
        };

        let mut record = NewRecord {
            account_id: self.account.id(),
            amount: *amount,
            currency: self.account.currency(),
            operation_date: self.command.operation_date()?,
            value_date: self.command.value_date()?,
            direction: *direction,
            mode: mode.clone(),
            details: description.join(" "),
            category_id: self
                .command
                .category(self.db)?
                .flatten()
                .as_ref()
                .and_then(Entity::id),
            merchant_id: self
                .command
                .merchant(self.db)?
                .flatten()
                .as_ref()
                .and_then(Entity::id),
        };

        record.save(self.db)?;
        Ok(())
    }

    fn list(&mut self) -> Result<()> {
        let RecordCommands::List {
            operation_date,
            greater_than,
            less_than,
            direction,
            mode,
            details,
            count,
            ..
        } = &self.command
        else {
            anyhow::bail!("wrong command passed: {:?}", self.command);
        };

        let criteria = QueryRecord {
            account_id: self.account.id(),
            after: self.command.after()?,
            before: self.command.before()?,
            operation_date: *operation_date,
            greater_than: greater_than.map(|m| m.into()),
            less_than: less_than.map(|m| m.into()),
            direction: *direction,
            mode: mode.clone(),
            details: details.clone(),
            count: *count,
            category_id: self
                .command
                .category(self.db)?
                .as_ref()
                .map(|c| c.as_ref().and_then(Entity::id)),
            merchant_id: self
                .command
                .merchant(self.db)?
                .as_ref()
                .map(|m| m.as_ref().and_then(Entity::id)),
        };

        criteria.for_each(self.db, |record| {
            println!("{:?}", record)
        })?;

        Ok(())
    }
}
