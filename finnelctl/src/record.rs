use anyhow::Result;

use crate::cli::{record::*, Commands};
use crate::config::Config;
use crate::record::display::RecordToDisplay;

use finnel::{
    prelude::*,
    record::{
        change::{ChangeRecord, ResolvedChangeRecord, ViolatingChangeRecord},
        NewRecord, QueryRecord,
    },
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
    let Commands::Record(command) = config.command().clone() else {
        anyhow::bail!("wrong command passed: {:?}", config.command());
    };

    let conn = &mut config.database()?;
    let mut cmd = CommandContext {
        account: config.account_or_default(conn)?,
        conn,
        _config: config,
    };

    match &command {
        Command::List(args) => cmd.list(args),
        Command::Show(args) => cmd.show(args),
        Command::Create(args) => cmd.create(args),
        Command::Update(args) => cmd.update(args),
        Command::Import(args) => cmd.import(args),
    }
}

impl CommandContext<'_> {
    fn list(&mut self, args: &List) -> Result<()> {
        let List {
            operation_date,
            greater_than,
            less_than,
            direction,
            mode,
            count,
            ..
        } = args;
        let details = args.details();

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
            order: args
                .sort
                .clone()
                .into_iter()
                .map(|o| o.into())
                .collect::<Vec<_>>(),
        };

        match &args.action {
            Some(ListAction::Update(args)) => {
                let (category, merchant) = relations_args(self.conn, args)?;
                let resolved_changes = change_args(self.conn, args, &category, &merchant)?;

                for (record, _, _) in query.run(self.conn)? {
                    resolved_changes
                        .validate(self.conn, &record)?
                        .save(self.conn)?;
                }
            }
            Some(ListAction::Delete { confirm }) => {
                self.conn.transaction(|conn| {
                    if !confirm || !crate::utils::confirm()? {
                        anyhow::bail!("operation requires confirmation");
                    }
                    for (mut record, _, _) in query.run(conn)? {
                        record.delete(conn)?;
                    }
                    Ok(())
                })?;
            }
            None => {
                let records = query
                    .run(self.conn)?
                    .into_iter()
                    .map(RecordToDisplay::from)
                    .collect::<Vec<_>>();

                println!("{}", Table::new(records));
            }
        }

        Ok(())
    }

    fn show(&mut self, args: &Show) -> Result<()> {
        let record = Record::find(self.conn, args.id())?;
        Ok(())
    }

    fn create(&mut self, args: &Create) -> Result<()> {
        let Create {
            amount,
            details,
            direction,
            mode,
            ..
        } = args;

        NewRecord {
            amount: *amount,
            operation_date: args.operation_date()?,
            value_date: args.value_date()?,
            direction: *direction,
            mode: *mode,
            details: details.as_str(),
            category: args.category(self.conn)?.as_ref(),
            merchant: args.merchant(self.conn)?.as_ref(),
            ..NewRecord::new(&self.account)
        }
        .save(self.conn)?;

        Ok(())
    }

    fn update(&mut self, args: &Update) -> Result<()> {
        let record = Record::find(self.conn, args.id())?;

        let (category, merchant) = relations_args(self.conn, &args.args)?;
        change_args(self.conn, &args.args, &category, &merchant)?
            .validate(self.conn, &record)?
            .save(self.conn)
            .optional_empty_changeset()?;

        Ok(())
    }

    fn import(&mut self, args: &Import) -> Result<()> {
        import::run(self.conn, &self.account, args)?;

        Ok(())
    }
}

fn relations_args<'a>(
    conn: &mut Conn,
    args: &'a UpdateArgs,
) -> Result<(Option<Option<Category>>, Option<Option<Merchant>>)> {
    let category = args.category(conn)?;
    let merchant = args.merchant(conn)?;

    Ok((category, merchant))
}

fn change_args<'a>(
    conn: &mut Conn,
    args: &'a UpdateArgs,
    category: &'a Option<Option<Category>>,
    merchant: &'a Option<Option<Merchant>>,
) -> Result<ResolvedChangeRecord<'a>> {
    Ok(if args.confirm {
        if !crate::utils::confirm()? {
            anyhow::bail!("operation requires confirmation");
        }

        ViolatingChangeRecord {
            amount: args.amount,
            operation_date: args.operation_date()?,
            value_date: args.value_date()?,
            direction: args.direction,
            mode: args.mode,
            details: args.details.as_deref(),
            category: category.as_ref().map(|o| o.as_ref()),
            merchant: merchant.as_ref().map(|o| o.as_ref()),
        }
        .into_resolved(conn)?
    } else {
        ChangeRecord {
            value_date: args.value_date()?,
            details: args.details.as_deref(),
            category: category.as_ref().map(|o| o.as_ref()),
            merchant: merchant.as_ref().map(|o| o.as_ref()),
        }
        .into_resolved(conn)?
    })
}
