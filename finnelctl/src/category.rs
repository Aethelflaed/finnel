use anyhow::Result;
use std::borrow::Cow;

use finnel::{
    category::{
        change::{ChangeCategory, ResolvedChangeCategory},
        NewCategory, QueryCategory,
    },
    prelude::*,
    record::QueryRecord,
};

use crate::cli::{category::*, Commands};
use crate::config::Config;
use crate::record::display::RecordToDisplay;

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

pub fn run(config: &Config) -> Result<()> {
    let Commands::Category(command) = config.command().clone() else {
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

        let query = QueryCategory {
            name: name.as_deref(),
            count: count.map(|c| c as i64),
            ..Default::default()
        };

        if let Some(ListUpdate::Update(args)) = &args.update {
            let (parent, replaced_by) = relations_args(self.conn, args)?;
            let resolved_changes = change_args(self.conn, args, &parent, &replaced_by)?;

            for category in query.run(self.conn)? {
                resolved_changes
                    .validate(self.conn, &category)?
                    .save(self.conn)?;
            }
        } else {
            let categories = query
                .with_parent()
                .with_replacer()
                .run(self.conn)?
                .into_iter()
                .map(CategoryToDisplay::from)
                .collect::<Vec<_>>();

            println!("{}", Table::new(categories));
        }

        Ok(())
    }

    fn show(&mut self, args: &Show) -> Result<()> {
        let category = Category::find_by_name(self.conn, &args.name)?;

        println!("{} | {}", category.id, category.name);

        println!();
        if let Ok(account) = self.config.account_or_default(self.conn) {
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
        } else {
            println!("Specify an account to see associated records");
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
        let category = Category::find_by_name(self.conn, &args.name)?;

        let (parent, replaced_by) = relations_args(self.conn, &args.args)?;
        change_args(self.conn, &args.args, &parent, &replaced_by)?
            .validate(self.conn, &category)?
            .save(self.conn)
            .optional_empty_changeset()?;

        Ok(())
    }

    fn delete(&mut self, args: &Delete) -> Result<()> {
        let mut category = Category::find_by_name(self.conn, &args.name)?;

        if args.confirm && crate::utils::confirm()? {
            category.delete(self.conn)?;
        } else {
            anyhow::bail!("operation requires confirmation");
        }

        Ok(())
    }
}

fn relations_args<'a>(
    conn: &mut Conn,
    args: &'a UpdateArgs,
) -> Result<(Option<Option<Category>>, Option<Option<Category>>)> {
    Ok((args.parent(conn)?, args.replace_by(conn)?))
}

fn change_args<'a>(
    conn: &mut Conn,
    args: &'a UpdateArgs,
    parent: &'a Option<Option<Category>>,
    replaced_by: &'a Option<Option<Category>>,
) -> Result<ResolvedChangeCategory<'a>> {
    Ok(ChangeCategory {
        name: args.new_name.as_deref(),
        parent: parent.as_ref().map(|o| o.as_ref()),
        replaced_by: replaced_by.as_ref().map(|o| o.as_ref()),
    }
    .into_resolved(conn)?)
}
