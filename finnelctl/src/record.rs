use anyhow::Result;

use crate::cli::{record::*, Commands};
use crate::config::Config;
use crate::record::display::RecordToDisplay;

use finnel::{
    prelude::*,
    record::{ChangeRecord, NewRecord, QueryRecord},
};

use tabled::Table;

pub mod display;
mod import;

struct CommandContext<'a> {
    _config: &'a Config,
    conn: &'a mut Database,
    account: Account,
}

pub fn run(config: &Config) -> Result<()> {
    let Commands::Record(command) = config.command().clone().unwrap() else {
        anyhow::bail!("wrong command passed: {:?}", config.command());
    };

    let conn = &mut config.database()?;
    let mut cmd = CommandContext {
        account: config.account_or_default(conn)?,
        conn,
        _config: config,
    };

    match &command {
        Command::Add(args) => cmd.add(args),
        Command::Update(args) => cmd.update(args),
        Command::List(args) => cmd.list(args),
        Command::Import(args) => cmd.import(args),
    }
}

impl CommandContext<'_> {
    fn add(&mut self, args: &Add) -> Result<()> {
        let Add {
            amount,
            details,
            direction,
            mode,
            ..
        } = args;

        let record = NewRecord {
            amount: *amount,
            operation_date: args.operation_date()?,
            value_date: args.value_date()?,
            direction: *direction,
            mode: *mode,
            details: details.as_str(),
            category_id: args
                .category(self.conn)?
                .flatten()
                .as_ref()
                .map(|c| c.id),
            merchant_id: args
                .merchant(self.conn)?
                .flatten()
                .as_ref()
                .map(|m| m.id),
            ..NewRecord::new(&self.account)
        };

        record.save(self.conn)?;
        Ok(())
    }

    fn update(&mut self, args: &Update) -> Result<()> {
        let mut record = Record::find(self.conn, args.id())?;

        args_to_change(self.conn, &args.args)?.apply(self.conn, &mut record)?;

        Ok(())
    }

    fn list(&mut self, args: &List) -> Result<()> {
        let List {
            operation_date,
            greater_than,
            less_than,
            direction,
            mode,
            details,
            count,
            ..
        } = args;

        let query = QueryRecord {
            account_id: Some(self.account.id),
            after: args.after()?,
            before: args.before()?,
            operation_date: *operation_date,
            greater_than: *greater_than,
            less_than: *less_than,
            direction: *direction,
            mode: *mode,
            details: details.as_deref(),
            category_id: args.category(self.conn)?.map(|c| c.map(|c| c.id)),
            merchant_id: args.merchant(self.conn)?.map(|m| m.map(|m| m.id)),
            count: *count,
        };

        if let Some(ListUpdate::Update(args)) = &args.update {
            let resolved_args = args_to_change(self.conn, args)?;

            for (record, _, _) in query.run(self.conn)? {
                resolved_args.clone().save(self.conn, &record)?;
            }
        } else {
            let records = query
                .run(self.conn)?
                .into_iter()
                .map(RecordToDisplay::from)
                .collect::<Vec<_>>();

            println!("{}", Table::new(records));
        }

        Ok(())
    }

    fn import(&mut self, args: &Import) -> Result<()> {
        let Import { file, profile, .. } = args;

        import::import(profile, file)?.persist(&self.account, self.conn)?;

        Ok(())
    }
}

fn args_to_change<'a>(
    conn: &mut Conn,
    args: &'a UpdateArgs,
) -> Result<ChangeRecord<'a>> {
    if args.confirm && !crate::utils::confirm()? {
        anyhow::bail!("operation requires confirmation");
    }

    Ok(ChangeRecord {
        amount: args.amount,
        operation_date: args.operation_date()?,
        value_date: args.value_date()?,
        direction: args.direction,
        mode: args.mode,
        details: args.details.as_deref(),
        category_id: args.category(conn)?.map(|c| c.map(|c| c.id)),
        merchant_id: args.merchant(conn)?.map(|m| m.map(|m| m.id)),
    })
}
