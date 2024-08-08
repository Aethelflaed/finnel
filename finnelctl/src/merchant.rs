use anyhow::{Context, Result};
use std::cell::OnceCell;

use finnel::{
    account::QueryAccount,
    merchant::{
        change::{ChangeMerchant, ResolvedChangeMerchant},
        NewMerchant, QueryMerchant,
    },
    prelude::*,
    record::QueryRecord,
};

use crate::cli::merchant::*;
use crate::config::Config;
use crate::record::display::RecordToDisplay;
use crate::utils::DeferrableResolvedUpdateArgs;

use tabled::{builder::Builder as TableBuilder, settings::Panel, Table};

struct CommandContext<'a> {
    config: &'a Config,
    conn: &'a mut Database,
}

pub fn run(config: &Config, command: &Command) -> Result<()> {
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

        match &args.action {
            Some(Action::Update(args)) => {
                let changes = ResolvedUpdateArgs::deferred(args);

                for merchant in query.run(self.conn)? {
                    changes
                        .get(self.conn)?
                        .validate(self.conn, &merchant)?
                        .save(self.conn)?;
                }
            }
            Some(Action::Delete { confirm }) => {
                if !confirm || !crate::utils::confirm()? {
                    anyhow::bail!("operation requires confirmation");
                }
                self.conn.transaction(|conn| {
                    for mut merchant in query.run(conn)? {
                        merchant.delete(conn)?;
                    }
                    Result::<()>::Ok(())
                })?;
            }
            None => {
                let mut builder = TableBuilder::new();
                push_record!(builder, "id", "name", "default category", "replaced by");
                for (merchant, default_category, replacer) in
                    query.with_replacer().with_category().run(self.conn)?
                {
                    push_record!(
                        builder,
                        merchant.id,
                        merchant.name,
                        default_category.map(|c| c.name),
                        replacer.map(|m| m.name),
                    );
                }

                println!("{}", builder.build());
            }
        }

        Ok(())
    }

    fn show(&mut self, args: &Show) -> Result<()> {
        //let mut merchant = Merchant::find_by_name(self.conn, &args.name)?;
        let mut merchant = args.identifier.find(self.conn)?;

        match &args.action {
            Some(Action::Update(args)) => {
                let changes = ResolvedUpdateArgs::deferred(args);

                changes
                    .get(self.conn)?
                    .validate(self.conn, &merchant)?
                    .save(self.conn)?;
            }
            Some(Action::Delete { confirm }) => {
                if !confirm || !crate::utils::confirm()? {
                    anyhow::bail!("operation requires confirmation");
                }
                self.conn.transaction(|conn| merchant.delete(conn))?;
            }
            None => {
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
                if let Ok(Some(account)) = self.config.account_or_default(self.conn) {
                    self.show_merchant_records(&merchant, &account)?;
                } else {
                    for account in QueryAccount::default().run(self.conn)? {
                        self.show_merchant_records(&merchant, &account)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn show_merchant_records(&mut self, merchant: &Merchant, account: &Account) -> Result<()> {
        let records = QueryRecord {
            account_id: Some(account.id),
            merchant_id: Some(Some(merchant.id)),
            ..Default::default()
        }
        .with_category()
        .with_merchant()
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
        let merchant = args.identifier.find(self.conn)?;

        ResolvedUpdateArgs::new(self.conn, &args.args)?
            .get(self.conn)?
            .validate(self.conn, &merchant)?
            .save(self.conn)
            .optional_empty_changeset()?;

        Ok(())
    }

    fn delete(&mut self, args: &Delete) -> Result<()> {
        let mut merchant = args.identifier.find(self.conn)?;

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

impl<'a> DeferrableResolvedUpdateArgs<'a, UpdateArgs, ResolvedChangeMerchant<'a>>
    for ResolvedUpdateArgs<'a>
{
    fn new(conn: &mut Conn, args: &'a UpdateArgs) -> Result<Self> {
        Ok(Self {
            args,
            default_category: args.default_category(conn)?,
            replaced_by: args.replace_by(conn)?,
            change_args: Default::default(),
        })
    }

    fn get(&'a self, conn: &mut Conn) -> Result<&ResolvedChangeMerchant<'a>> {
        #[allow(clippy::collapsible_if)]
        if self.change_args.get().is_none() {
            if self
                .change_args
                .set(
                    ChangeMerchant {
                        name: self.args.new_name.as_deref(),
                        default_category: self.default_category.as_ref().map(|o| o.as_ref()),
                        replaced_by: self.replaced_by.as_ref().map(|o| o.as_ref()),
                    }
                    .into_resolved(conn)?,
                )
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
