use anyhow::{Context, Result};
use std::borrow::Borrow;
use std::cell::OnceCell;

use crate::cli::record::*;
use crate::config::Config;
use crate::utils::DeferrableResolvedUpdateArgs;

use finnel::{
    prelude::*,
    record::{
        change::{ChangeRecord, ResolvedChangeRecord, ViolatingChangeRecord},
        NewRecord, QueryRecord, SplitRecord,
    },
};

use tabled::builder::Builder as TableBuilder;

struct CommandContext<'a> {
    config: &'a Config,
    conn: &'a mut Database,
    account: Option<Account>,
}

pub fn run(config: &Config, command: &Command) -> Result<()> {
    let conn = &mut config.database()?;
    let mut cmd = CommandContext {
        account: config.account_or_default(conn)?,
        conn,
        config,
    };

    match &command {
        Command::List(args) => cmd.list(args),
        Command::Show(args) => cmd.show(args),
        Command::Create(args) => cmd.create(args),
        Command::Update(args) => cmd.update(args),
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

        let mut order = args
            .sort
            .clone()
            .into_iter()
            .map(|o| o.into())
            .collect::<Vec<_>>();

        if order.is_empty() {
            if let Some(sort) = self.configuration(ConfigurationKey::DefaultSort)? {
                let sort = Sort::try_from(&sort)?;
                order.push(sort.into());
            }
        }

        let query = QueryRecord {
            account_id: self.account.as_ref().map(|a| a.id),
            from: args.from,
            to: args.to,
            operation_date: *operation_date,
            greater_than: *greater_than,
            less_than: *less_than,
            direction: *direction,
            mode: *mode,
            details: details.as_deref(),
            category_id: args.category(self.conn)?.map(|c| c.map(|c| c.id)),
            merchant_id: args.merchant(self.conn)?.map(|m| m.map(|m| m.id)),
            count: *count,
            order,
            ..QueryRecord::default()
        };

        use ListAction::*;

        match &args.action {
            Some(Other(Action::Update(args))) => {
                let changes = ResolvedUpdateArgs::deferred(args);

                for record in query.run(self.conn)? {
                    changes
                        .get(self.conn)?
                        .validate(self.conn, &record)?
                        .save(self.conn)?;
                }
            }
            Some(Other(Action::Delete { confirm })) => {
                if !confirm || !crate::utils::confirm()? {
                    anyhow::bail!("operation requires confirmation");
                }
                self.conn.transaction(|conn| {
                    for mut record in query.run(conn)? {
                        record.delete(conn)?;
                    }
                    Result::<()>::Ok(())
                })?;
            }
            Some(Config(config)) => {
                self.configure(config)?;
            }
            None => {
                if self.account.is_some() {
                    table_display!(query
                        .with_category()
                        .with_parent()
                        .with_merchant()
                        .run(self.conn)?);
                } else {
                    table_display!(query
                        .with_account()
                        .with_category()
                        .with_parent()
                        .with_merchant()
                        .run(self.conn)?);
                }
            }
        }

        Ok(())
    }

    fn configure(&mut self, config: &ConfigurationAction) -> Result<()> {
        use ConfigurationAction::*;
        use ConfigurationKey::*;

        match config {
            Get { key } => {
                if let Some(value) = self.configuration(key)? {
                    println!("{}", value);
                }
            }
            Set { key, value } => {
                let value = match key {
                    DefaultSort => Sort::try_from(value)?.to_string(),
                };
                self.config
                    .set(format!("records/{}", key.as_str()).as_str(), value.as_str())?;
            }
            Reset { key } => {
                self.config
                    .reset(format!("records/{}", key.as_str()).as_str())?;
            }
        }

        Ok(())
    }

    fn show(&mut self, args: &Show) -> Result<()> {
        let mut record = Record::find(self.conn, args.id())?;

        use ShowAction::*;

        match &args.action {
            Some(Other(Action::Update(args))) => {
                let changes = ResolvedUpdateArgs::deferred(args);

                changes
                    .get(self.conn)?
                    .validate(self.conn, &record)?
                    .save(self.conn)?;
            }
            Some(Other(Action::Delete { confirm })) => {
                if !confirm || !crate::utils::confirm()? {
                    anyhow::bail!("operation requires confirmation");
                }
                record.delete(self.conn)?;
            }
            Some(Split(args)) => {
                SplitRecord {
                    amount: args.amount,
                    details: args.details.as_deref(),
                    category: args.category(self.conn)?.as_ref().map(|c| c.as_ref()),
                }
                .save(self.conn, &record)?;
            }
            None => {
                let category = record.fetch_category(self.conn)?;
                let merchant = record.fetch_merchant(self.conn)?;

                let mut builder = TableBuilder::new();
                table_push_row!(
                    builder,
                    std::marker::PhantomData::<(Record, Option<Category>, Option<Merchant>)>
                );
                table_push_row!(builder, (record, category, merchant));

                println!("{}", builder.build());
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

        let Some(account) = self.account.as_ref() else {
            anyhow::bail!("Account not provided")
        };

        NewRecord {
            amount: *amount,
            operation_date: args.operation_date(),
            value_date: args.value_date(),
            direction: *direction,
            mode: *mode,
            details: details.as_str(),
            category: args.category(self.conn)?.as_ref(),
            merchant: args.merchant(self.conn)?.as_ref(),
            ..NewRecord::new(account)
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

    fn configuration<T>(&self, key: T) -> Result<Option<String>>
    where
        T: Borrow<ConfigurationKey>,
    {
        self.config
            .get(format!("records/{}", key.borrow().as_str()).as_str())
    }
}

struct ResolvedUpdateArgs<'a> {
    args: &'a UpdateArgs,
    category: Option<Option<Category>>,
    merchant: Option<Option<Merchant>>,
    change_args: OnceCell<ResolvedChangeRecord<'a>>,
}

impl<'a> DeferrableResolvedUpdateArgs<'a, UpdateArgs, ResolvedChangeRecord<'a>>
    for ResolvedUpdateArgs<'a>
{
    fn new(conn: &mut Conn, args: &'a UpdateArgs) -> Result<Self> {
        Ok(Self {
            args,
            category: args.category(conn)?,
            merchant: args.merchant(conn)?,
            change_args: Default::default(),
        })
    }

    fn get(&'a self, conn: &mut Conn) -> Result<&ResolvedChangeRecord<'a>> {
        #[allow(clippy::collapsible_if)]
        if self.change_args.get().is_none() {
            if self
                .change_args
                .set(if self.args.confirm {
                    if !crate::utils::confirm()? {
                        anyhow::bail!("operation requires confirmation");
                    }

                    ViolatingChangeRecord {
                        amount: self.args.amount,
                        operation_date: self.args.operation_date,
                        value_date: self.args.value_date,
                        direction: self.args.direction,
                        mode: self.args.mode,
                        details: self.args.details.as_deref(),
                        category: self.category.as_ref().map(|o| o.as_ref()),
                        merchant: self.merchant.as_ref().map(|o| o.as_ref()),
                    }
                    .into_resolved(conn)?
                } else {
                    ChangeRecord {
                        value_date: self.args.value_date,
                        details: self.args.details.as_deref(),
                        category: self.category.as_ref().map(|o| o.as_ref()),
                        merchant: self.merchant.as_ref().map(|o| o.as_ref()),
                    }
                    .into_resolved(conn)?
                })
                .is_err()
            {
                anyhow::bail!("Failed to set supposedly empty OnceCell");
            }
        }
        self.change_args
            .get()
            .context("Failed to get supposedly initialized OnceCell")
    }
}
