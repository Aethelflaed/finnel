use std::path::PathBuf;

use clap::{Parser, Subcommand};

/// Finnel control
#[derive(Default, Clone, Debug, Parser)]
#[command(version)]
pub struct Cli {
    #[clap(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,

    /// Sets a custom config directory
    ///
    /// The default value is $FINNEL_CONFIG if it is set, or
    /// $XDG_CONFIG_HOME/finnel otherwise
    #[arg(short = 'C', long, value_name = "DIR")]
    pub config: Option<PathBuf>,

    /// Sets a custom data directory
    ///
    /// The default value is $FINNEL_DATA if it is set, or
    /// $XDG_DATA_HOME/finnel otherwise
    #[arg(short = 'D', long, value_name = "DIR")]
    pub data: Option<PathBuf>,

    /// Sets the account to consider for the following command
    ///
    /// A default value can be configured
    #[arg(short = 'A', long, value_name = "ACCOUNT")]
    pub account: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Commands {
    Account {
        #[command(subcommand)]
        command: AccountCommands,
    },
    Record {},
}

#[derive(Debug, Clone, Subcommand)]
pub enum AccountCommands {
    /// List registered accounts
    List {},
    /// Create a new account
    Create {
        /// Name of the new account
        account_name: String,
    },
    /// Show details about an account
    Show {},
    /// Delete an account
    Delete {
        /// Confirm deletion
        #[arg(long, hide = true)]
        confirm: bool,
    },
    /// Check or set the default account
    Default {
        /// Reset the default account
        #[arg(short, long)]
        reset: bool,
    },
}
