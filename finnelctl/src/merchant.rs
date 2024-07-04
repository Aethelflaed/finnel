use anyhow::Result;
use std::borrow::Cow;

use finnel::{merchant::QueryMerchant, Database, Entity, Merchant, Query};

use crate::cli::{merchant::*, Commands};
use crate::config::Config;

use tabled::{Table, Tabled};

struct CommandContext<'a> {
    _config: &'a Config,
    db: &'a mut Database,
}

#[derive(derive_more::From)]
struct MerchantToDisplay(Merchant);

impl Tabled for MerchantToDisplay {
    const LENGTH: usize = 2;

    fn fields(&self) -> Vec<Cow<'_, str>> {
        vec![self.id(), self.0.name.clone().into()]
    }

    fn headers() -> Vec<Cow<'static, str>> {
        vec!["id".into(), "name".into()]
    }
}

impl MerchantToDisplay {
    fn id(&self) -> Cow<'_, str> {
        if let Some(id) = self.0.id() {
            id.value().to_string().into()
        } else {
            Default::default()
        }
    }
}

pub fn run(config: &Config) -> Result<()> {
    let Commands::Merchant(command) = config.command().clone().unwrap() else {
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

        let query = QueryMerchant {
            name: name.clone(),
            count: *count,
            ..Default::default()
        };

        let mut merchants = Vec::<MerchantToDisplay>::new();

        for merchant in query.statement(&self.db)?.iter()? {
            merchants.push(merchant?.into());
        }

        println!("{}", Table::new(merchants));

        Ok(())
    }
}
