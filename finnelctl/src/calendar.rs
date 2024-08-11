use anyhow::Result;

use finnel::{prelude::*, record::QueryRecord};

use crate::cli::calendar::*;
use crate::config::Config;

use chrono::{prelude::*, Days, Months};

use tabled::{builder::Builder as TableBuilder, settings::Panel};

struct CommandContext<'a> {
    #[allow(dead_code)]
    config: &'a Config,
    conn: &'a mut Database,
}

pub fn run(config: &Config, command: &Command) -> Result<()> {
    let conn = &mut config.database()?;
    let mut cmd = CommandContext { conn, config };

    match &command {
        Command::Show(args) => cmd.show(args),
        Command::Today(args) => cmd.today(args),
    }
}

impl CommandContext<'_> {
    fn today(&mut self, _args: &Today) -> Result<()> {
        let today = Utc::now().date_naive();
        let tomorrow = today + Days::new(1);

        let query = QueryRecord {
            from: Some(today),
            to: Some(tomorrow),
            ..QueryRecord::default()
        }
        .with_account()
        .with_category()
        .with_parent()
        .with_merchant();

        let mut builder = TableBuilder::new();
        table_push_row!(builder, query.type_marker());
        for result in query.run(self.conn)? {
            table_push_row!(builder, result);
        }

        println!("{}", builder.build());

        Ok(())
    }

    fn show(&mut self, _args: &Show) -> Result<()> {
        let date = Utc::now();
        let month = Month::try_from(u8::try_from(date.month())?)?;
        let start_of_month = NaiveDate::from_ymd_opt(date.year(), date.month(), 1)
            .ok_or(anyhow::anyhow!("Cannot compute start of month"))?;
        let end_of_month = start_of_month
            .checked_add_months(Months::new(1))
            .ok_or(anyhow::anyhow!("Cannot add 1 month"))?
            .checked_sub_days(Days::new(1))
            .ok_or(anyhow::anyhow!("Cannot sub 1 day"))?;

        println!("{}", start_of_month);
        println!("{}", end_of_month);

        let mut builder = TableBuilder::new();
        table_push_row_elements!(
            builder,
            "Monday",
            "Tuesday",
            "Wednesday",
            "Thursday",
            "Friday",
            "Saturday",
            "Sunday"
        );

        let offset = start_of_month.weekday().num_days_from_monday() - 1;
        let days = end_of_month.day() + offset;
        let number_of_weeks = days / 7 + u32::from(days % 7 != 0);
        let days_of_month = (0..number_of_weeks)
            .map(|week| {
                Ok((0..7)
                    .map(|day_of_week| {
                        let index = week * 7 + day_of_week;
                        if index <= offset || index > days {
                            Ok(None)
                        } else {
                            Ok(Some(
                                NaiveDate::from_ymd_opt(date.year(), date.month(), index - offset)
                                    .ok_or(anyhow::anyhow!(
                                        "Cannot compute day {}",
                                        index - offset
                                    ))?,
                            ))
                        }
                    })
                    .collect::<Result<Vec<Option<NaiveDate>>>>()?)
            })
            .collect::<Result<Vec<_>>>()?;

        for week in days_of_month {
            table_push_row_elements!(
                builder, week[0], week[1], week[2], week[3], week[4], week[5], week[6],
            );
        }

        println!();

        println!("{}", builder.build().with(Panel::header(month.name())));

        Ok(())
    }
}
