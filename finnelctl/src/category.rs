use anyhow::Result;
use std::borrow::Cow;

use finnel::{category::QueryCategory, Database, Entity, Category, Query};

use crate::cli::{category::*, Commands};
use crate::config::Config;

use tabled::{Table, Tabled};

struct CommandContext<'a> {
    _config: &'a Config,
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
    let mut cmd = CommandContext {
        db,
        _config: config,
    };

    match &command {
        Command::List(args) => cmd.list(args),
        _ => todo!(),
    }
}

impl CommandContext<'_> {
    fn list(&mut self, args: &List) -> Result<()> {
        let List {
            name,
            count,
            ..
        } = args;

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
}
