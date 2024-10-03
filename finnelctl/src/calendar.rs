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
    stats_retriever: StatsRetriever,
}

pub fn run(config: &Config, args: &Arguments) -> Result<()> {
    let conn = &mut config.database()?;
    let categories = args.categories(conn)?;
    let mut cmd = CommandContext {
        conn,
        config,
        stats_retriever: StatsRetriever {
            categories,
            direction: args.direction,
        }
    };

    match &args.command.clone().unwrap_or_default() {
        Command::Month(args) => cmd.month(args),
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

    fn month(&mut self, args: &Monthly) -> Result<()> {
        let month = args.calendar_month()?.build(self.conn, &self.stats_retriever)?;
        println!("{}", month);

        Ok(())
    }
}

struct StatsRetriever {
    categories: Option<Vec<Category>>,
    direction: Option<Direction>,
}

impl StatsRetriever {
    pub fn get(&self, conn: &mut Conn, range: Range<NaiveDate>) -> Result<Stats> {
        let stats =
            CategoriesStats::from_date_range_and_currency(conn, range, Currency::EUR)?.0;

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

#[derive(Default)]
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

pub struct CalendarMonth {
    pub start_of_month: NaiveDate,
    days: Vec<Vec<Option<CalendarDay>>>,
    stats: Stats,
}

impl CalendarMonth {
    fn month(&self) -> Result<Month> {
        Ok(Month::try_from(u8::try_from(self.start_of_month.month())?)?)
    }

    fn build(mut self, conn: &mut Conn, retriever: &StatsRetriever) -> Result<Self> {
        let start_of_month = self.start_of_month;
        let end_of_month = start_of_month + Months::new(1) - Days::new(1);

        let offset = start_of_month.weekday().num_days_from_monday();
        let days = end_of_month.day() + offset;
        let number_of_weeks = days / 7 + u32::from(days % 7 != 0);

        self.days = (0..number_of_weeks)
            .map(|week| {
                (0..7).map(|day_of_week| {
                    // We add 1 because days are 1..=7 not 0..=6
                    let index = week * 7 + day_of_week + 1;
                    if index <= offset || index > days {
                        Ok(None)
                    } else {
                        let date = NaiveDate::from_ymd_opt(
                            start_of_month.year(),
                            start_of_month.month(),
                            index - offset,
                        )
                            .ok_or(anyhow::anyhow!("Cannot compute day {}", index - offset))?;
                        Ok(Some(CalendarDay::new(
                                    date,
                                    retriever.get(conn, date..(date + Days::new(1)))?,
                        )))
                    }
                })
                .collect::<Result<Vec<Option<CalendarDay>>>>()
            })
            .collect::<Result<_>>()?;

        self.stats = retriever.get(conn, start_of_month..end_of_month)?;

        Ok(self)
    }
}

impl TryFrom<NaiveDate> for CalendarMonth {
    type Error = anyhow::Error;

    fn try_from(start_of_month: NaiveDate) -> Result<Self> {
        if start_of_month.day() != 1 {
            anyhow::bail!(
                "Cannot create calendar month with non start-of-month day {}",
                start_of_month
            );
        }
        Ok(CalendarMonth {
            start_of_month,
            days: Default::default(),
            stats: Default::default(),
        })
    }
}

impl std::fmt::Display for CalendarMonth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

        for week in &self.days {
            table_push_row_elements!(
                builder, week[0], week[1], week[2], week[3], week[4], week[5], week[6],
            );
        }
        writeln!(f, "{}",
            builder
                .build()
                .with(Panel::header(self.month().unwrap().name()))
                .with(Panel::footer(format!(
                    "Debit: {}\nCredit: {}",
                    self.stats.debit_amount(),
                    self.stats.credit_amount()
                )))
        )
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
