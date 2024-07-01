use anyhow::Result;

use crate::cli::{Commands, RecordCommands};
use crate::config::Config;

use finnel::{
    record::{NewRecord, QueryRecord},
    Account, Database, Entity, Query,
};

use tabled::Table;

mod display;
mod import;

struct RecordCmd<'a> {
    _config: &'a Config,
    db: &'a mut Database,
    account: Account,
    command: RecordCommands,
}

pub fn run(config: &Config) -> Result<()> {
    let Commands::Record { command } = config.command().clone().unwrap() else {
        anyhow::bail!("wrong command passed: {:?}", config.command());
    };

    let db = &mut config.database()?;
    let mut cmd = RecordCmd {
        account: config.account_or_default(db)?,
        db,
        _config: config,
        command: command.clone(),
    };

    match command {
        RecordCommands::Add { .. } => cmd.add(),
        RecordCommands::List { .. } => cmd.list(),
        RecordCommands::Import { .. } => cmd.import(),
    }
}

impl RecordCmd<'_> {
    fn add(&mut self) -> Result<()> {
        let RecordCommands::Add {
            amount,
            details,
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
            details: details.clone(),
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

        let query = QueryRecord {
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

        println!("{}", query.query());
        for (key, value) in query.params() {
            println!("{} => {:?}", key, value.to_sql()?);
        }

        let mut records = Vec::<display::RecordToDisplay>::new();
        query.for_each(self.db, |record| records.push(record.into()))?;

        println!("{}", Table::new(records));

        Ok(())
    }

    fn import(&mut self) -> Result<()> {
        let RecordCommands::Import { file, profile, .. } = &self.command else {
            anyhow::bail!("wrong command passed: {:?}", self.command);
        };

        import::import(profile, file)?.persist(&self.account, self.db)?;

        Ok(())
    }
}
