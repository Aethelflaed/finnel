use anyhow::Result;
use std::borrow::Cow;

use finnel::{
    category::{ChangeCategory, NewCategory, QueryCategory},
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
struct CategoryToDisplay(Category);

impl Tabled for CategoryToDisplay {
    const LENGTH: usize = 2;

    fn fields(&self) -> Vec<Cow<'_, str>> {
        vec![self.0.id.to_string().into(), self.0.name.clone().into()]
    }

    fn headers() -> Vec<Cow<'static, str>> {
        vec!["id".into(), "name".into()]
    }
}

pub fn run(config: &Config) -> Result<()> {
    let Commands::Category(command) = config.command().clone().unwrap() else {
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
        let List { name, count, .. } = args;

        let categories = QueryCategory {
            name: name.as_deref(),
            count: count.map(|c| c as i64),
        }
        .run(self.conn)?
        .into_iter()
        .map(CategoryToDisplay::from)
        .collect::<Vec<_>>();

        println!("{}", Table::new(categories));

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
        NewCategory::new(&args.name).save(self.conn)?;
        Ok(())
    }

    fn update(&mut self, args: &Update) -> Result<()> {
        let category = Category::find_by_name(self.conn, &args.name)?;

        ChangeCategory {
            name: args.new_name.as_deref(),
        }
        .save(self.conn, &category)
        .optional_empty_changeset()?;

        Ok(())
    }

    fn delete(&mut self, args: &Delete) -> Result<()> {
        let mut category = Category::find_by_name(self.conn, &args.name)?;

        if args.confirm {
            category.delete(self.conn)?;
        } else {
            anyhow::bail!("operation requires confirmation flag");
        }

        Ok(())
    }
}
