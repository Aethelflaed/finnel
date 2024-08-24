use anyhow::Result;

use clap::{Args, Subcommand};

use crate::cli::category::Identifier as CategoryIdentifier;
use crate::cli::report::Identifier as ReportIdentifier;
use finnel::prelude::*;

#[derive(Args, Clone, Debug)]
pub struct Arguments {
    #[command(subcommand)]
    pub command: Command,

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
    /// Show the calendar
    Show(Show),
    /// Show report for today
    Today(Today),
}

#[derive(Default, Args, Clone, Debug)]
pub struct Show {}

#[derive(Default, Args, Clone, Debug)]
pub struct Today {}
