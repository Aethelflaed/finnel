use anyhow::{Context, Result};
use std::cell::OnceCell;

use finnel::{
    category::{
        change::{ChangeCategory, ResolvedChangeCategory},
        NewCategory, QueryCategory,
    },
    prelude::*,
    record::QueryRecord,
};

use crate::cli::{category::*, record::Sort};
use crate::config::Config;
use crate::utils::DeferrableResolvedUpdateArgs;

use tabled::builder::Builder as TableBuilder;

struct CommandContext<'a> {
    #[allow(dead_code)]
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
                let mut builder = TableBuilder::new();
                table_push_row_elements!(builder, "id", "name", "parent", "replaced by");

                for (category, parent, replacer) in
                    query.with_parent().with_replacer().run(self.conn)?
                {
                    table_push_row_elements!(builder, category.id, category.name, parent, replacer);
                }

                println!("{}", builder.build());
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
                let mut ids = vec![category.id];
                println!("{} | {}", category.id, category.name);

                if let Some(parent) = category.fetch_parent(self.conn)? {
                    ids.push(parent.id);
                    println!("  Parent: {} | {}", parent.id, parent.name);
                }
                if let Some(replaced_by) = category.fetch_replaced_by(self.conn)? {
                    println!("  Replaced by: {} | {}", replaced_by.id, replaced_by.name);
                }

                let mut builder = TableBuilder::new();
                table_push_row_elements!(builder, "id", "name", "replaced by");
                for (child, replacer) in (QueryCategory {
                    parent_id: Some(Some(category.id)),
                    ..QueryCategory::default()
                })
                .with_replacer()
                .run(self.conn)?
                {
                    ids.push(child.id);
                    table_push_row_elements!(builder, child.id, child.name, replacer);
                }

                if builder.count_records() > 1 {
                    println!("Children:\n{}", builder.build());
                }

                self.show_category_records(&ids)?;
            }
        }

        Ok(())
    }

    fn show_category_records(&mut self, ids: &Vec<i64>) -> Result<()> {
        println!();
        let query = QueryRecord {
            category_ids: Some(ids),
            order: vec![Sort::try_from("date.desc")?.into()],
            ..Default::default()
        }
        .with_account()
        .with_category()
        .with_merchant();

        let mut builder = TableBuilder::new();
        table_push_row!(builder, query.type_marker());
        for result in query.run(self.conn)? {
            table_push_row!(builder, result);
        }

        let count = builder.count_records() - 1;

        if count > 0 {
            println!("{}", builder.build());
        } else {
            println!("No associated records");
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
