use anyhow::Result;
use std::borrow::Cow;

use finnel::{
    account::{NewAccount, QueryAccount},
    prelude::*,
};

use crate::cli::{account::*, Commands};
use crate::config::Config;

use tabled::{Table, Tabled};

struct CommandContext<'a> {
    config: &'a Config,
    conn: &'a mut Database,
}

#[derive(derive_more::From)]
struct AccountToDisplay(Account);

impl Tabled for AccountToDisplay {
    const LENGTH: usize = 3;

    fn fields(&self) -> Vec<Cow<'_, str>> {
        vec![
            self.0.id.to_string().into(),
            self.0.name.clone().into(),
            self.0.balance().to_string().into(),
        ]
    }

    fn headers() -> Vec<Cow<'static, str>> {
        vec!["id".into(), "name".into(), "balance".into()]
    }
}

pub fn run(config: &Config) -> Result<()> {
    let Commands::Account(command) = config.command().clone().unwrap() else {
        anyhow::bail!("wrong command passed: {:?}", config.command());
    };

    let conn = &mut config.database()?;
    let mut cmd = CommandContext { conn, config };

    match &command {
        Command::List(args) => cmd.list(args),
        Command::Create(args) => cmd.create(args),
        Command::Show(args) => cmd.show(args),
        Command::Delete(args) => cmd.delete(args),
        Command::Default(args) => cmd.default(args),
    }
}

impl CommandContext<'_> {
    fn get(&mut self, name: Option<&str>) -> Result<Account> {
        Ok(if let Some(name) = name {
            Account::find_by_name(self.conn, name)?
        } else {
            self.config.account_or_default(self.conn)?
        })
    }

    fn list(&mut self, _args: &List) -> Result<()> {
        let accounts = QueryAccount::default()
            .run(self.conn)?
            .into_iter()
            .map(AccountToDisplay::from)
            .collect::<Vec<_>>();

        println!("{}", Table::new(accounts));

        Ok(())
    }

    fn show(&mut self, args: &Show) -> Result<()> {
        let account = self.get(args.name.as_deref())?;

        println!("{} | {}", account.id, account.name);
        println!("\tBalance: {}", account.balance());

        Ok(())
    }

    fn create(&mut self, args: &Create) -> Result<()> {
        NewAccount::new(&args.name).save(self.conn)?;
        Ok(())
    }

    fn delete(&mut self, args: &Delete) -> Result<()> {
        let mut account = self.get(args.name.as_deref())?;

        if args.confirm {
            account.delete(self.conn)?;
        } else {
            anyhow::bail!("operation requires confirmation flag");
        }
        Ok(())
    }

    fn default(&mut self, args: &Default) -> Result<()> {
        if let Some(name) = args.name.as_deref().or(self.config.account_name())
        {
            let account = Account::find_by_name(self.conn, name)?;
            Ok(self.config.set("default_account", &account.name)?)
        } else if args.reset {
            Ok(self.config.reset("default_account")?)
        } else {
            let account_name = self
                .config
                .default_account(self.conn)?
                .map(|a| a.name.clone())
                .unwrap_or("<not set>".to_string());
            println!("{}", account_name);
            Ok(())
        }
    }
}
