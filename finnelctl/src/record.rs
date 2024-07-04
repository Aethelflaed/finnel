use anyhow::Result;

use crate::cli::{record::*, Commands};
use crate::config::Config;

use finnel::{
    record::{NewRecord, QueryRecord},
    Account, Category, Connection, Database, Entity, Merchant, Query, Record,
};

use chrono::{DateTime, Utc};
use tabled::Table;

mod display;
mod import;

struct CommandContext<'a> {
    _config: &'a Config,
    db: &'a mut Database,
    account: Account,
}

pub fn run(config: &Config) -> Result<()> {
    let Commands::Record(command) = config.command().clone().unwrap() else {
        anyhow::bail!("wrong command passed: {:?}", config.command());
    };

    let db = &mut config.database()?;
    let mut cmd = CommandContext {
        account: config.account_or_default(db)?,
        db,
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

        let mut record = NewRecord {
            account_id: self.account.id(),
            amount: *amount,
            currency: self.account.currency(),
            operation_date: args.operation_date()?,
            value_date: args.value_date()?,
            direction: *direction,
            mode: mode.clone(),
            details: details.clone(),
            category_id: args
                .category(self.db)?
                .flatten()
                .as_ref()
                .and_then(Entity::id),
            merchant_id: args
                .merchant(self.db)?
                .flatten()
                .as_ref()
                .and_then(Entity::id),
        };

        record.save(self.db)?;
        Ok(())
    }

    fn update(&mut self, args: &Update) -> Result<()> {
        let mut record = Record::find(self.db, args.id())?;

        self.update_record(
            &mut record,
            &ResolvedUpdateArgs::try_from(self.db, &args.args)?,
        )
    }

    fn update_record(
        &self,
        record: &mut Record,
        args: &ResolvedUpdateArgs,
    ) -> Result<()> {
        if let Some(details) = args.details.clone() {
            record.details = details;
        }
        if let Some(date) = args.value_date {
            record.value_date = date;
        }
        if let Some(category) = &args.category {
            record.set_category(category.as_ref());
        }
        if let Some(merchant) = &args.merchant {
            record.set_merchant(merchant.as_ref());
        }

        record.save(self.db)?;

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
            account_id: self.account.id(),
            after: args.after()?,
            before: args.before()?,
            operation_date: *operation_date,
            greater_than: greater_than.map(|m| m.into()),
            less_than: less_than.map(|m| m.into()),
            direction: *direction,
            mode: mode.clone(),
            details: details.clone(),
            count: *count,
            category_id: args
                .category(self.db)?
                .as_ref()
                .map(|c| c.as_ref().and_then(Entity::id)),
            merchant_id: args
                .merchant(self.db)?
                .as_ref()
                .map(|m| m.as_ref().and_then(Entity::id)),
        };

        println!("{:?}", query);

        if let Some(ListUpdate::Update(args)) = &args.update {
            let resolved_args = ResolvedUpdateArgs::try_from(self.db, &args)?;

            for record in query.statement(&self.db)?.iter()? {
                self.update_record(&mut record?.record, &resolved_args)?;
            }
        } else {
            let mut records = Vec::<display::RecordToDisplay>::new();

            for record in query.statement(&self.db)?.iter()? {
                records.push(record?.into());
            }

            println!("{}", Table::new(records));
        }

        Ok(())
    }

    fn import(&mut self, args: &Import) -> Result<()> {
        let Import { file, profile, .. } = args;

        import::import(profile, file)?.persist(&self.account, self.db)?;

        Ok(())
    }
}

struct ResolvedUpdateArgs {
    details: Option<String>,
    value_date: Option<DateTime<Utc>>,
    category: Option<Option<Category>>,
    merchant: Option<Option<Merchant>>,
}

impl ResolvedUpdateArgs {
    fn try_from(db: &Connection, args: &UpdateArgs) -> Result<Self> {
        Ok(ResolvedUpdateArgs {
            details: args.details.clone(),
            value_date: args.value_date()?,
            category: args.category(&db)?,
            merchant: args.merchant(&db)?,
        })
    }
}
