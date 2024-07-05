use anyhow::Result;
use std::borrow::Cow;

use finnel::{
    category::QueryCategory, record::QueryRecord, Category, Database, Entity,
    Query,
};

use crate::cli::{category::*, Commands};
use crate::config::Config;
use crate::record::display;

use tabled::{settings::Panel, Table, Tabled};

struct CommandContext<'a> {
    config: &'a Config,
    db: &'a mut Database,
}

#[derive(derive_more::From)]
struct CategoryToDisplay(Category);

impl Tabled for CategoryToDisplay {
    const LENGTH: usize = 2;

    fn fields(&self) -> Vec<Cow<'_, str>> {
        vec![self.id(), self.0.name.clone().into()]
    }

    fn headers() -> Vec<Cow<'static, str>> {
        vec!["id".into(), "name".into()]
    }
}

impl CategoryToDisplay {
    fn id(&self) -> Cow<'_, str> {
        if let Some(id) = self.0.id() {
            id.value().to_string().into()
        } else {
            Default::default()
        }
    }
}

pub fn run(config: &Config) -> Result<()> {
    let Commands::Category(command) = config.command().clone().unwrap() else {
        anyhow::bail!("wrong command passed: {:?}", config.command());
    };

    let db = &mut config.database()?;
    let mut cmd = CommandContext { db, config };

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

        let query = QueryCategory {
            name: name.clone(),
            count: *count,
            ..Default::default()
        };

        let mut categories = Vec::<CategoryToDisplay>::new();

        for category in query.statement(&self.db)?.iter()? {
            categories.push(category?.into());
        }

        println!("{}", Table::new(categories));

        Ok(())
    }

    fn show(&mut self, args: &Show) -> Result<()> {
        let category = Category::find_by_name(self.db, &args.name)?;

        println!("{} | {}", category.id().unwrap().value(), category.name);

        println!();
        if let Ok(account) = self.config.account_or_default(self.db) {
            let query = QueryRecord {
                account_id: account.id(),
                category_id: Some(category.id()),
                ..Default::default()
            };

            let mut records = Vec::<display::RecordToDisplay>::new();

            for record in query.statement(&self.db)?.iter()? {
                records.push(record?.into());
            }
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
        let mut category = Category::new(args.name.clone());

        category.save(&self.db)?;

        Ok(())
    }

    fn update(&mut self, args: &Update) -> Result<()> {
        let mut category = Category::find_by_name(self.db, &args.name)?;

        if let Some(name) = args.new_name.clone() {
            category.name = name;
        }

        category.save(&self.db)?;

        Ok(())
    }

    fn delete(&mut self, args: &Delete) -> Result<()> {
        let mut category = Category::find_by_name(self.db, &args.name)?;

        if args.confirm {
            category.delete(&mut self.db)?;
        } else {
            anyhow::bail!("operation requires confirmation flag");
        }

        Ok(())
    }
}
