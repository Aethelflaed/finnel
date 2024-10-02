use crate::calendar::CalendarMonth;
use crate::cli::category::Identifier as CategoryIdentifier;
use crate::cli::report::Identifier as ReportIdentifier;
use anyhow::Result;
use clap::{Args, Subcommand};
use finnel::prelude::*;

#[derive(Args, Clone, Debug)]
pub struct Arguments {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Show reports stats instead of global stats
    #[arg(
        long,
        global = true,
        help_heading = "Filter stats",
        group = "reports_or_categories"
    )]
    report: Option<ReportIdentifier>,

    /// Show only stats for given categories, by id or name, separated by comma
    #[arg(
        long,
        global = true,
        help_heading = "Filter stats",
        group = "reports_or_categories",
        value_delimiter = ','
    )]
    categories: Option<Vec<CategoryIdentifier>>,

    /// Show only stats for the given direction (credit or debit)
    #[arg(long, global = true, help_heading = "Filter stats")]
    pub direction: Option<Direction>,
}

impl Arguments {
    pub fn categories(&self, conn: &mut Conn) -> Result<Option<Vec<Category>>> {
        if let Some(id) = &self.report {
            return Ok(Some(id.find(conn)?.categories));
        } else if let Some(ids) = &self.categories {
            return Ok(Some(
                ids.iter()
                    .map(|id| id.find(conn))
                    .collect::<Result<Vec<_>>>()?,
            ));
        }
        Ok(None)
    }
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// Show report for today
    Today(Today),
    /// Show monthly view
    Month(Monthly),
}

impl Default for Command {
    fn default() -> Self {
        Command::Month(Monthly::default())
    }
}

#[derive(Default, Args, Clone, Debug)]
pub struct Today {}

#[derive(Default, Args, Clone, Debug)]
pub struct Monthly {
    /// Show previous month
    #[arg(
        short,
        long,
        alias = "prev",
        group = "month_arg",
        help_heading = "Month options"
    )]
    pub previous: bool,

    /// Show next month
    #[arg(short, long, group = "month_arg", help_heading = "Month options")]
    pub next: bool,

    /// Show given month, either name or number, or even YYYY/mmm
    #[arg(group = "month_arg", help_heading = "Month options")]
    pub month: Option<String>,
}

impl Monthly {
    pub fn calendar_month(&self) -> Result<CalendarMonth> {
        #[cfg(not(test))]
        use chrono::Utc;
        #[cfg(test)]
        use tests::Utc;
        use anyhow::Context;
        use chrono::{Datelike, Month, Months, NaiveDate};

        let today = Utc::now().date_naive();
        let mut start_of_month = NaiveDate::from_ymd_opt(today.year(), today.month(), 1)
            .ok_or(anyhow::anyhow!("Cannot compute start of month"))?;

        if self.previous {
            start_of_month = start_of_month - Months::new(1);
        } else if self.next {
            start_of_month = start_of_month + Months::new(1);
        } else if let Some(month_name) = self.month.as_deref() {
            let (year, month) = if let Some((year, month)) = month_name.split_once("/") {
                (year.parse::<i32>()?, month)
            } else {
                (today.year(), month_name)
            };

            let month = month
                .parse::<u32>()
                .or_else(|_| month.parse::<Month>().map(|m| m.number_from_month()))
                .with_context(|| format!("Parsing from {:?}", month))?;

            start_of_month = NaiveDate::from_ymd_opt(year, month, 1).ok_or(anyhow::anyhow!(
                "Cannot compute start of month for {}",
                month_name
            ))?;
        }

        start_of_month.try_into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::prelude::{assert_eq, Result};
    use chrono::NaiveDate;

    macro_rules! monthly {
        (previous) => {
            Monthly {
                previous: true,
                ..Monthly::default()
            }
        };
        (next) => {
            Monthly {
                next: true,
                ..Monthly::default()
            }
        };
        ($date:literal) => {
            Monthly {
                month: Some($date.to_string()),
                ..Monthly::default()
            }
        };
        ($date:expr) => {
            Monthly {
                month: Some($date),
                ..Monthly::default()
            }
        };
    }

    pub struct Utc;
    impl Utc {
        pub fn now() -> chrono::DateTime<chrono::Utc> {
            NaiveDate::from_ymd_opt(2024, 9, 10)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc()
        }
    }

    #[test]
    fn calendar_month() -> Result<()> {
        assert_eq!(
            NaiveDate::from_ymd_opt(2024, 9, 1).unwrap(),
            Monthly::default().calendar_month()?.start_of_month
        );
        assert_eq!(
            NaiveDate::from_ymd_opt(2024, 8, 1).unwrap(),
            monthly!(previous).calendar_month()?.start_of_month
        );
        assert_eq!(
            NaiveDate::from_ymd_opt(2024, 10, 1).unwrap(),
            monthly!(next).calendar_month()?.start_of_month
        );

        assert_eq!(
            NaiveDate::from_ymd_opt(2025, 09, 1).unwrap(),
            monthly!("2025/Sep").calendar_month()?.start_of_month
        );
        assert_eq!(
            NaiveDate::from_ymd_opt(2026, 09, 1).unwrap(),
            monthly!("2026/september").calendar_month()?.start_of_month
        );
        assert_eq!(
            NaiveDate::from_ymd_opt(2024, 09, 1).unwrap(),
            monthly!("2024/09").calendar_month()?.start_of_month
        );
        assert_eq!(
            NaiveDate::from_ymd_opt(2024, 10, 1).unwrap(),
            monthly!("2024/10").calendar_month()?.start_of_month
        );

        assert_eq!(
            NaiveDate::from_ymd_opt(2024, 10, 1).unwrap(),
            monthly!("oct").calendar_month()?.start_of_month
        );
        assert_eq!(
            NaiveDate::from_ymd_opt(2024, 10, 1).unwrap(),
            monthly!("october").calendar_month()?.start_of_month
        );
        Ok(())
    }
}
