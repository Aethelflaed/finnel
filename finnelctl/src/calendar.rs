use anyhow::Result;
use std::ops::Range;

use finnel::{
    prelude::*,
    record::QueryRecord,
    stats::{CategoriesStats, CategoryStats},
};

use crate::cli::calendar::*;
use crate::config::Config;

use chrono::{prelude::*, Days, Months};

use tabled::{builder::Builder as TableBuilder, settings::Panel};

struct CommandContext<'a> {
    #[allow(dead_code)]
    config: &'a Config,
    conn: &'a mut Database,
    categories: Option<Vec<Category>>,
    direction: Option<Direction>,
}

pub fn run(config: &Config, args: &Arguments) -> Result<()> {
    let conn = &mut config.database()?;
    let categories = args.categories(conn)?;
    let mut cmd = CommandContext {
        conn,
        config,
        categories,
        direction: args.direction,
    };

    match &args.command {
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
        let today = Utc::now().date_naive();
        let tomorrow = today + Days::new(1);

        let month = Month::try_from(u8::try_from(today.month())?)?;
        let start_of_month = NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
            .ok_or(anyhow::anyhow!("Cannot compute start of month"))?;
        let end_of_month = start_of_month + Months::new(1) - Days::new(1);

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
                (0..7)
                    .map(|day_of_week| {
                        let index = week * 7 + day_of_week;
                        if index <= offset || index > days {
                            Ok(None)
                        } else {
                            let date = NaiveDate::from_ymd_opt(
                                today.year(),
                                today.month(),
                                index - offset,
                            )
                            .ok_or(anyhow::anyhow!("Cannot compute day {}", index - offset))?;
                            Ok(Some(CalendarDay::new(
                                date,
                                self.retrieve_stats(date..(date + Days::new(1)))?,
                            )))
                        }
                    })
                    .collect::<Result<Vec<Option<CalendarDay>>>>()
            })
            .collect::<Result<Vec<_>>>()?;

        for week in days_of_month {
            table_push_row_elements!(
                builder, week[0], week[1], week[2], week[3], week[4], week[5], week[6],
            );
        }

        println!();

        let stats = self.retrieve_stats(start_of_month..tomorrow)?;

        println!(
            "{}",
            builder
                .build()
                .with(Panel::header(month.name()))
                .with(Panel::footer(format!(
                    "Debit: {}\nCredit: {}",
                    stats.debit_amount(),
                    stats.credit_amount()
                )))
        );

        Ok(())
    }

    fn retrieve_stats(&mut self, range: Range<NaiveDate>) -> Result<Stats> {
        let stats =
            CategoriesStats::from_date_range_and_currency(self.conn, range, Currency::EUR)?.0;

        Ok(stats
            .into_iter()
            .filter(|stats| {
                self.direction
                    .as_ref()
                    .map(|dir| stats.direction == *dir)
                    .unwrap_or(true)
                    && self
                        .categories
                        .as_ref()
                        .map(|cats| cats.iter().any(|cat| Some(cat.id) == stats.category_id))
                        .unwrap_or(true)
            })
            .collect::<Vec<_>>()
            .into())
    }
}

struct Stats {
    debit_amount: Decimal,
    credit_amount: Decimal,
}

impl From<Vec<CategoryStats>> for Stats {
    fn from(stats: Vec<CategoryStats>) -> Self {
        Self {
            debit_amount: stats
                .iter()
                .filter(|stats| stats.direction.is_debit())
                .fold(Decimal::ZERO, |acc, e| acc + e.amount),
            credit_amount: stats
                .iter()
                .filter(|stats| stats.direction.is_credit())
                .fold(Decimal::ZERO, |acc, e| acc + e.amount),
        }
    }
}

impl Stats {
    pub fn debit_amount(&self) -> Amount {
        Amount(self.debit_amount, Currency::EUR)
    }

    pub fn credit_amount(&self) -> Amount {
        Amount(self.credit_amount, Currency::EUR)
    }
}

struct CalendarDay {
    date: NaiveDate,
    stats: Stats,
}

impl CalendarDay {
    pub fn new(date: NaiveDate, stats: Stats) -> Self {
        CalendarDay { date, stats }
    }
}

impl std::fmt::Display for CalendarDay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.date.day())?;
        writeln!(f, "{}", self.stats.debit_amount())?;
        writeln!(f, "{}", self.stats.credit_amount())?;

        Ok(())
    }
}

impl crate::utils::table_display::RowElementDisplay for Option<CalendarDay> {
    fn to_row_element(&self) -> String {
        self.as_ref().map(|d| d.to_string()).unwrap_or_default()
    }
}
