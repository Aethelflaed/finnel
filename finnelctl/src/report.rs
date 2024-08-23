use anyhow::Result;

use finnel::prelude::*;

use crate::cli::report::*;
use crate::config::Config;

use tabled::builder::Builder as TableBuilder;

struct CommandContext<'a> {
    _config: &'a Config,
    conn: &'a mut Database,
}

pub fn run(config: &Config, command: &Command) -> Result<()> {
    let conn = &mut config.database()?;
    let mut cmd = CommandContext {
        conn,
        _config: config,
    };

    match &command {
        Command::List(args) => cmd.list(args),
        Command::Show(args) => cmd.show(args),
        Command::Create(args) => cmd.create(args),
        Command::Delete(args) => cmd.delete(args),
    }
}

impl CommandContext<'_> {
    fn list(&mut self, _args: &List) -> Result<()> {
        let mut builder = TableBuilder::new();
        table_push_row_elements!(builder, "id", "name");

        for (id, name) in Report::all(self.conn)? {
            table_push_row_elements!(builder, id, name);
        }

        println!("{}", builder.build());

        Ok(())
    }

    fn show(&mut self, args: &Show) -> Result<()> {
        let mut report = args.identifier.find(self.conn)?;

        match &args.action {
            Some(Action::Add { categories }) => {
                let categories = categories
                    .iter()
                    .map(|id| id.find(self.conn))
                    .collect::<Result<Vec<_>>>()?;
                report.add(self.conn, categories.iter())?;
            }
            Some(Action::Remove { categories }) => {
                let categories = categories
                    .iter()
                    .map(|id| id.find(self.conn))
                    .collect::<Result<Vec<_>>>()?;
                report.remove(self.conn, categories.iter())?;
            }
            None => {
                println!("{} | {}", report.id, report.name);

                let mut builder = TableBuilder::new();
                table_push_row_elements!(builder, "id", "name");
                for category in &report.categories {
                    table_push_row_elements!(builder, category.id, category.name);
                }
                println!("{}", builder.build());
            }
        }

        Ok(())
    }

    fn create(&mut self, args: &Create) -> Result<()> {
        Report::create(self.conn, &args.name)?;
        Ok(())
    }

    fn delete(&mut self, args: &Delete) -> Result<()> {
        let mut report = args.identifier.find(self.conn)?;

        if args.confirm && crate::utils::confirm()? {
            report.delete(self.conn)?;
        } else {
            anyhow::bail!("operation requires confirmation");
        }
        Ok(())
    }
}
