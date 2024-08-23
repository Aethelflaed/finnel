use std::path::PathBuf;

use chrono::NaiveDate;
use clap::Args;

#[derive(Default, Args, Clone, Debug)]
pub struct Command {
    /// File to import
    #[arg(help_heading = "Import")]
    pub file: PathBuf,

    /// Import profile to use
    #[arg(short = 'P', long, help_heading = "Import")]
    pub profile: String,

    /// Print importer records
    #[arg(long, help_heading = "Import")]
    pub print: bool,

    /// Do not persist any of the imported records
    #[arg(long, help_heading = "Import")]
    pub pretend: bool,

    /// Only import records with an operation date greater than or equal to this one
    #[arg(long, value_name = "DATE", help_heading = "Filter records")]
    pub from: Option<NaiveDate>,

    /// Only import records with an operation date less than or equal to this one
    #[arg(long, value_name = "DATE", help_heading = "Filter records")]
    pub to: Option<NaiveDate>,
}
