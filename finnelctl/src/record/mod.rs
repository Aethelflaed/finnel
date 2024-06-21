use anyhow::Result;

use crate::cli::{Commands, RecordCommands};
use crate::config::Config;

use finnel::{account::NewRecord, Account, Database, Entity};

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
            account: self.account.id(),
            amount: *amount,
            currency: self.account.currency(),
            operation_date: self.command.operation_date()?,
            value_date: self.command.value_date()?,
            direction: *direction,
            mode: mode.clone(),
            details: description.join(" "),
            category: self.command.category(self.db)?.and_then(|c| c.id()),
            merchant: self.command.merchant(self.db)?.and_then(|m| m.id()),
        };

        record.save(self.db)?;
        Ok(())
    }
}
