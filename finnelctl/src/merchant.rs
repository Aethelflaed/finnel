use anyhow::{Context, Result};
use std::borrow::Cow;
use std::cell::OnceCell;

use finnel::{
    merchant::{
        change::{ChangeMerchant, ResolvedChangeMerchant},
        NewMerchant, QueryMerchant,
    },
    prelude::*,
    record::QueryRecord,
};

use crate::cli::{merchant::*, Commands};
use crate::config::Config;
use crate::record::display::RecordToDisplay;

use tabled::{settings::Panel, Table, Tabled};

struct CommandContext<'a> {
    config: &'a Config,
    conn: &'a mut Database,
}

#[derive(derive_more::From)]
struct MerchantToDisplay(Merchant, Option<Category>, Option<Merchant>);

impl Tabled for MerchantToDisplay {
    const LENGTH: usize = 2;

    fn fields(&self) -> Vec<Cow<'_, str>> {
        vec![
            self.0.id.to_string().into(),
            self.0.name.clone().into(),
            self.1
                .as_ref()
                .map(|c| c.name.clone().into())
                .unwrap_or("".into()),
            self.2
                .as_ref()
                .map(|c| c.name.clone().into())
                .unwrap_or("".into()),
        ]
    }

    fn headers() -> Vec<Cow<'static, str>> {
        vec![
            "id".into(),
            "name".into(),
            "default category".into(),
            "replaced by".into(),
        ]
    }
}

pub fn run(config: &Config) -> Result<()> {
    let Commands::Merchant(command) = config.command().clone() else {
        anyhow::bail!("wrong command passed: {:?}", config.command());
    };

    let conn = &mut config.database()?;
    let mut cmd = CommandContext { conn, config };

    match &command {
        Command::List(args) => cmd.list(args),
        Command::Create(args) => cmd.create(args),
        Command::Update(args) => cmd.update(args),
        Command::Show(args) => cmd.show(args),
        Command::Delete(args) => cmd.delete(args),
    }
}

impl CommandContext<'_> {
    fn list(&mut self, args: &List) -> Result<()> {
        let List { count, .. } = args;
        let name = args.name();

        let query = QueryMerchant {
            name: name.as_deref(),
            count: count.map(|c| c as i64),
        };

        if let Some(ListAction::Update(args)) = &args.action {
            let changes = DeferredUpdateArgsResolution::new(args);

            for merchant in query.run(self.conn)? {
                changes
                    .get(self.conn)?
                    .validate(self.conn, &merchant)?
                    .save(self.conn)?;
            }
        } else {
            let merchants = query
                .with_replacer()
                .with_category()
                .run(self.conn)?
                .into_iter()
                .map(MerchantToDisplay::from)
                .collect::<Vec<_>>();

            println!("{}", Table::new(merchants));
        }

        Ok(())
    }

    fn show(&mut self, args: &Show) -> Result<()> {
        let merchant = Merchant::find_by_name(self.conn, &args.name)?;

        println!("{} | {}", merchant.id, merchant.name);

        if let Some(default_category) = merchant.fetch_default_category(self.conn)? {
            println!(
                "  Default category: {} | {}",
                default_category.id, default_category.name
            );
        }
        if let Some(replaced_by) = merchant.fetch_replaced_by(self.conn)? {
            println!("  Replaced by: {} | {}", replaced_by.id, replaced_by.name);
        }

        println!();
        if let Ok(account) = self.config.account_or_default(self.conn) {
            let records = QueryRecord {
                account_id: Some(account.id),
                merchant_id: Some(Some(merchant.id)),
                ..Default::default()
            }
            .run(self.conn)?
            .into_iter()
            .map(RecordToDisplay::from)
            .collect::<Vec<_>>();

            let count = records.len();

            if count > 0 {
                println!(
                    "{}",
                    Table::new(records).with(Panel::header(format!(
                        "{} associated records for account {}",
                        count, account.name
                    )))
                );
            } else {
                println!("No associated records for account {}", account.name);
            }
        } else {
            println!("Specify an account to see associated records");
        }

        Ok(())
    }

    fn create(&mut self, args: &Create) -> Result<()> {
        NewMerchant {
            name: &args.name,
            default_category: args.default_category(self.conn)?.as_ref(),
            replaced_by: args.replace_by(self.conn)?.as_ref(),
        }
        .save(self.conn)?;

        Ok(())
    }

    fn update(&mut self, args: &Update) -> Result<()> {
        let merchant = Merchant::find_by_name(self.conn, &args.name)?;

        ResolvedUpdateArgs::new(self.conn, &args.args)?
            .get(self.conn)?
            .validate(self.conn, &merchant)?
            .save(self.conn)
            .optional_empty_changeset()?;

        Ok(())
    }

    fn delete(&mut self, args: &Delete) -> Result<()> {
        let mut merchant = Merchant::find_by_name(self.conn, &args.name)?;

        if args.confirm && crate::utils::confirm()? {
            merchant.delete(self.conn)?;
        } else {
            anyhow::bail!("operation requires confirmation");
        }

        Ok(())
    }
}

struct ResolvedUpdateArgs<'a> {
    args: &'a UpdateArgs,
    default_category: Option<Option<Category>>,
    replaced_by: Option<Option<Merchant>>,
    change_args: OnceCell<ResolvedChangeMerchant<'a>>,
}

impl<'a> ResolvedUpdateArgs<'a> {
    pub fn new(conn: &mut Conn, args: &'a UpdateArgs) -> Result<Self> {
        Ok(Self {
            args: args,
            default_category: args.default_category(conn)?,
            replaced_by: args.replace_by(conn)?,
            change_args: Default::default(),
        })
    }

    pub fn get(&'a self, conn: &mut Conn) -> Result<&ResolvedChangeMerchant<'a>> {
        if self.change_args.get().is_none() {
            match self.change_args.set(
                ChangeMerchant {
                    name: self.args.new_name.as_deref(),
                    default_category: self.default_category.as_ref().map(|o| o.as_ref()),
                    replaced_by: self.replaced_by.as_ref().map(|o| o.as_ref()),
                }
                .into_resolved(conn)?,
            ) {
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

    pub fn get(&'a self, conn: &mut Conn) -> Result<&ResolvedChangeMerchant<'a>> {
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
