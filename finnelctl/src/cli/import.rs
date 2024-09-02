use chrono::NaiveDate;
use clap::{Args, Subcommand, ValueEnum};

#[derive(Default, Args, Clone, Debug)]
pub struct Command {
    #[command(subcommand)]
    pub configuration_action: Option<ConfigurationAction>,

    /// File to import
    #[arg(help_heading = "Import")]
    pub file: Option<String>,

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

#[derive(Subcommand, Clone, Debug)]
pub enum ConfigurationAction {
    /// Print the configuration value
    Get {
        key: ConfigurationKey,
    },
    /// Set the configuration value
    Set {
        key: ConfigurationKey,
        value: String,
    },
    /// Remove the configuration value
    Reset {
        key: ConfigurationKey,
    },
}

#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum ConfigurationKey {
    DefaultAccount,
    DefaultFile,
}

impl ConfigurationKey {
    pub fn as_str(&self) -> &str {
        use ConfigurationKey::*;
        match self {
            DefaultAccount => "default_account",
            DefaultFile => "default_file",
        }
    }
}
