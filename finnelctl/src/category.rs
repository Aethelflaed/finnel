use anyhow::{Context, Result};
use std::borrow::Cow;
use std::cell::OnceCell;

use finnel::{
    account::QueryAccount,
    category::{
        change::{ChangeCategory, ResolvedChangeCategory},
        NewCategory, QueryCategory,
    },
    prelude::*,
    record::QueryRecord,
};

use crate::cli::category::*;
use crate::config::Config;
use crate::record::display::RecordToDisplay;
use crate::utils::DeferrableResolvedUpdateArgs;

use tabled::{settings::Panel, Table, Tabled};

struct CommandContext<'a> {
    config: &'a Config,
    conn: &'a mut Database,
}

#[derive(derive_more::From)]
struct CategoryToDisplay(Category, Option<Category>, Option<Category>);

impl Tabled for CategoryToDisplay {
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
            "parent".into(),
            "replaced by".into(),
        ]
    }
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

        let query = QueryCategory {
            name: name.as_deref(),
            count: count.map(|c| c as i64),
            ..Default::default()
        };

        match &args.action {
            Some(Action::Update(args)) => {
                let changes = ResolvedUpdateArgs::deferred(args);

                for category in query.run(self.conn)? {
                    changes
                        .get(self.conn)?
                        .validate(self.conn, &category)?
                        .save(self.conn)?;
                }
            }
            Some(Action::Delete { confirm }) => {
                if !confirm || !crate::utils::confirm()? {
                    anyhow::bail!("operation requires confirmation");
                }
                self.conn.transaction(|conn| {
                    for mut category in query.run(conn)? {
                        category.delete(conn)?;
                    }
                    Result::<()>::Ok(())
                })?;
            }
            None => {
                let categories = query
                    .with_parent()
                    .with_replacer()
                    .run(self.conn)?
                    .into_iter()
                    .map(CategoryToDisplay::from)
                    .collect::<Vec<_>>();

                println!("{}", Table::new(categories));
            }
        }

        Ok(())
    }

    fn show(&mut self, args: &Show) -> Result<()> {
        let mut category = args.identifier.find(self.conn)?;

        match &args.action {
            Some(Action::Update(args)) => {
                let changes = ResolvedUpdateArgs::deferred(args);

                changes
                    .get(self.conn)?
                    .validate(self.conn, &category)?
                    .save(self.conn)?;
            }
            Some(Action::Delete { confirm }) => {
                if !confirm || !crate::utils::confirm()? {
                    anyhow::bail!("operation requires confirmation");
                }
                self.conn.transaction(|conn| category.delete(conn))?;
            }
            None => {
                println!("{} | {}", category.id, category.name);

                if let Some(parent) = category.fetch_parent(self.conn)? {
                    println!("  Parent: {} | {}", parent.id, parent.name);
                }
                if let Some(replaced_by) = category.fetch_replaced_by(self.conn)? {
                    println!("  Replaced by: {} | {}", replaced_by.id, replaced_by.name);
                }

                println!();
                if let Ok(Some(account)) = self.config.account_or_default(self.conn) {
                    self.show_category_records(&category, &account)?;
                } else {
                    for account in QueryAccount::default().run(self.conn)? {
                        self.show_category_records(&category, &account)?;
                    }
                }
            }
        }

        Ok(())
    }

    fn show_category_records(&mut self, category: &Category, account: &Account) -> Result<()> {
        let records = QueryRecord {
            account_id: Some(account.id),
            category_id: Some(Some(category.id)),
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
        Ok(())
    }

    fn create(&mut self, args: &Create) -> Result<()> {
        NewCategory {
            name: &args.name,
            parent: args.parent(self.conn)?.as_ref(),
            replaced_by: args.replace_by(self.conn)?.as_ref(),
        }
        .save(self.conn)?;

        Ok(())
    }

    fn update(&mut self, args: &Update) -> Result<()> {
        let category = args.identifier.find(self.conn)?;

        ResolvedUpdateArgs::new(self.conn, &args.args)?
            .get(self.conn)?
            .validate(self.conn, &category)?
            .save(self.conn)
            .optional_empty_changeset()?;

        Ok(())
    }

    fn delete(&mut self, args: &Delete) -> Result<()> {
        let mut category = args.identifier.find(self.conn)?;

        if args.confirm && crate::utils::confirm()? {
            category.delete(self.conn)?;
        } else {
            anyhow::bail!("operation requires confirmation");
        }

        Ok(())
    }
}

struct ResolvedUpdateArgs<'a> {
    args: &'a UpdateArgs,
    parent: Option<Option<Category>>,
    replaced_by: Option<Option<Category>>,
    change_args: OnceCell<ResolvedChangeCategory<'a>>,
}

impl<'a> DeferrableResolvedUpdateArgs<'a, UpdateArgs, ResolvedChangeCategory<'a>>
    for ResolvedUpdateArgs<'a>
{
    fn new(conn: &mut Conn, args: &'a UpdateArgs) -> Result<Self> {
        Ok(Self {
            args,
            parent: args.parent(conn)?,
            replaced_by: args.replace_by(conn)?,
            change_args: Default::default(),
        })
    }

    fn get(&'a self, conn: &mut Conn) -> Result<&ResolvedChangeCategory<'a>> {
        #[allow(clippy::collapsible_if)]
        if self.change_args.get().is_none() {
            if self
                .change_args
                .set(
                    ChangeCategory {
                        name: self.args.new_name.as_deref(),
                        parent: self.parent.as_ref().map(|o| o.as_ref()),
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
