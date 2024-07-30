use anyhow::{Context, Result};
use std::cell::OnceCell;

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
            Some(Action::Update(args)) => {
                let changes = DeferredUpdateArgsResolution::new(args);

                for (record, _, _) in query.run(self.conn)? {
                    changes
                        .get(self.conn)?
                        .validate(self.conn, &record)?
                        .save(self.conn)?;
                }
            }
            Some(Action::Delete { confirm }) => {
                if !confirm || !crate::utils::confirm()? {
                    anyhow::bail!("operation requires confirmation");
                }
                self.conn.transaction(|conn| {
                    for (mut record, _, _) in query.run(conn)? {
                        record.delete(conn)?;
                    }
                    Result::<()>::Ok(())
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
        let mut record = Record::find(self.conn, args.id())?;

        match &args.action {
            Some(Action::Update(args)) => {
                let changes = DeferredUpdateArgsResolution::new(args);

                changes
                    .get(self.conn)?
                    .validate(self.conn, &record)?
                    .save(self.conn)?;
            }
            Some(Action::Delete { confirm }) => {
                if !confirm || !crate::utils::confirm()? {
                    anyhow::bail!("operation requires confirmation");
                }
                record.delete(self.conn)?;
            }
            None => {
                let category = record
                    .category_id
                    .map(|id| Category::find(self.conn, id))
                    .transpose()?;
                let merchant = record
                    .merchant_id
                    .map(|id| Merchant::find(self.conn, id))
                    .transpose()?;
                println!(
                    "{}",
                    Table::new(vec![RecordToDisplay::from((record, category, merchant,))])
                );
            }
        }
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

        ResolvedUpdateArgs::new(self.conn, &args.args)?
            .get(self.conn)?
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

struct ResolvedUpdateArgs<'a> {
    args: &'a UpdateArgs,
    category: Option<Option<Category>>,
    merchant: Option<Option<Merchant>>,
    change_args: OnceCell<ResolvedChangeRecord<'a>>,
}

impl<'a> ResolvedUpdateArgs<'a> {
    pub fn new(conn: &mut Conn, args: &'a UpdateArgs) -> Result<Self> {
        Ok(Self {
            args: args,
            category: args.category(conn)?,
            merchant: args.merchant(conn)?,
            change_args: Default::default(),
        })
    }

    pub fn get(&'a self, conn: &mut Conn) -> Result<&ResolvedChangeRecord<'a>> {
        if self.change_args.get().is_none() {
            match self.change_args.set(if self.args.confirm {
                if !crate::utils::confirm()? {
                    anyhow::bail!("operation requires confirmation");
                }

                ViolatingChangeRecord {
                    amount: self.args.amount,
                    operation_date: self.args.operation_date()?,
                    value_date: self.args.value_date()?,
                    direction: self.args.direction,
                    mode: self.args.mode,
                    details: self.args.details.as_deref(),
                    category: self.category.as_ref().map(|o| o.as_ref()),
                    merchant: self.merchant.as_ref().map(|o| o.as_ref()),
                }
                .into_resolved(conn)?
            } else {
                ChangeRecord {
                    value_date: self.args.value_date()?,
                    details: self.args.details.as_deref(),
                    category: self.category.as_ref().map(|o| o.as_ref()),
                    merchant: self.merchant.as_ref().map(|o| o.as_ref()),
                }
                .into_resolved(conn)?
            }) {
                Err(_) => anyhow::bail!("Failed to set supposedly empty OnceCell"),
                _ => {}
            }
        }
        self.change_args
            .get()
            .context("Failed to get supposedly initialized OnceCell")
    }
}

struct DeferredUpdateArgsResolution<'a> {
    args: &'a UpdateArgs,
    resolved_args: OnceCell<ResolvedUpdateArgs<'a>>,
}

impl<'a> DeferredUpdateArgsResolution<'a> {
    pub fn new(args: &'a UpdateArgs) -> Self {
        Self {
            args,
            resolved_args: Default::default(),
        }
    }

    pub fn get(&'a self, conn: &mut Conn) -> Result<&ResolvedChangeRecord<'a>> {
        if self.resolved_args.get().is_none() {
            match self
                .resolved_args
                .set(ResolvedUpdateArgs::new(conn, self.args)?)
            {
                Err(_) => anyhow::bail!("Failed to set supposedly empty OnceCell"),
                _ => {}
            }
        }
        self.resolved_args
            .get()
            .context("Failed to get supposedly initialized OnceCell")?
            .get(conn)
    }
}
