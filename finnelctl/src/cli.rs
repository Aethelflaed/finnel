use std::path::PathBuf;

use clap::{Parser, Subcommand};

macro_rules! create_identifier {
    ($struct:ty) => {
        #[derive(Args, Clone, Debug)]
        pub struct Identifier {
            /// Name or id
            pub name_or_id: String,
        }

        impl Identifier {
            pub fn find(&self, conn: &mut Conn) -> Result<$struct> {
                if self.name_or_id.chars().all(|c| c.is_ascii_digit()) {
                    Ok(<$struct>::find(conn, self.name_or_id.parse()?)?)
                } else {
                    Ok(<$struct>::find_by_name(conn, &self.name_or_id)?)
                }
            }
        }

        impl From<String> for Identifier {
            fn from(value: String) -> Self {
                Self { name_or_id: value }
            }
        }
    };
}

pub mod account;
pub mod calendar;
pub mod category;
pub mod import;
pub mod merchant;
pub mod record;
pub mod report;

/// Finnel control
#[derive(Default, Clone, Debug, Parser)]
#[command(version, infer_subcommands = true)]
pub struct Cli {
    #[clap(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,

    /// Sets a custom config directory
    ///
    /// The default value is $FINNEL_CONFIG if it is set, or
    /// $XDG_CONFIG_HOME/finnel otherwise
    #[arg(
        short = 'C',
        long,
        value_name = "DIR",
        global = true,
        help_heading = "Global options"
    )]
    pub config: Option<PathBuf>,

    /// Sets a custom data directory
    ///
    /// The default value is $FINNEL_DATA if it is set, or
    /// $XDG_DATA_HOME/finnel otherwise
    #[arg(
        short = 'D',
        long,
        value_name = "DIR",
        global = true,
        help_heading = "Global options"
    )]
    pub data: Option<PathBuf>,

    /// Sets the account to consider for the following command
    ///
    /// A default value can be configured
    #[arg(
        short = 'A',
        long,
        value_name = "NAME",
        global = true,
        help_heading = "Global options"
    )]
    pub account: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Subcommand)]
pub enum Commands {
    /// Account related commands
    #[command(subcommand)]
    Account(account::Command),
    /// Record related commands
    #[command(subcommand)]
    Record(record::Command),
    /// Category related commands
    #[command(subcommand)]
    Category(category::Command),
    /// Merchant related commands
    #[command(subcommand)]
    Merchant(merchant::Command),
    /// Display the calendar
    Calendar(calendar::Arguments),
    /// Configure reports
    #[command(subcommand)]
    Report(report::Command),
    /// Import records
    Import(import::Command),
    /// Consolidate the database
    Consolidate {},
    /// Reset the database
    #[command(hide = true)]
    Reset {
        #[arg(long, required = true)]
        confirm: bool,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Cli::command().debug_assert();
    }
}
