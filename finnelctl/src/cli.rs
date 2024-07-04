use std::path::PathBuf;

use clap::{Parser, Subcommand};

pub mod account;
pub mod record;

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
