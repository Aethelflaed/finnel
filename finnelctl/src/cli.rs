use std::path::PathBuf;

use finnel::{
    transaction::{Direction, Mode},
    Category, Connection, Decimal, Entity, Merchant,
};

use anyhow::Result;
use chrono::{offset::Utc, DateTime, NaiveDate, TimeZone};
use clap::{Args, Parser, Subcommand};

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

#[derive(Debug, Clone, Subcommand)]
pub enum Commands {
    /// Account related commands
    Account {
        #[command(subcommand)]
        command: AccountCommands,
    },
    /// Record related commands
    Record {
        #[command(subcommand)]
        command: RecordCommands,
    },
    /// Reset the database
    #[command(hide = true)]
    Reset {
        #[arg(long, required = true)]
        confirm: bool,
    },
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

#[derive(Debug, Clone, Subcommand)]
pub enum RecordCommands {
    /// Add a new record
    Add {
        /// Amount of the record
        ///
        /// Without currency symbol, the currency is inferred from the account
        amount: Decimal,

        /// Describe the record
        #[arg(required = true)]
        description: Vec<String>,

        /// Transaction direction
        ///
        /// Possible values include debit, credit, and variants
        #[arg(short = 'd', long, default_value_t = Direction::Debit, help_heading = "Record")]
        direction: Direction,

        /// Transaction mode
        ///
        /// Possible values include direct, transfer, ATM
        #[arg(short = 'm', long, default_value_t = Mode::Direct, help_heading = "Record")]
        mode: Mode,

        /// Operation date
        #[arg(long, value_name = "DATE", help_heading = "Record")]
        operation_date: Option<NaiveDate>,

        /// Value date
        #[arg(long, value_name = "DATE", help_heading = "Record")]
        value_date: Option<NaiveDate>,

        #[allow(private_interfaces)]
        #[command(flatten, next_help_heading = "Merchant")]
        merchant: MerchantArgs,

        #[allow(private_interfaces)]
        #[command(flatten, next_help_heading = "Category")]
        category: CategoryArgs,
    },
}

impl RecordCommands {
    pub fn merchant(&self, db: &Connection) -> Result<Option<Merchant>> {
        let arg = match self {
            RecordCommands::Add { merchant, .. } => merchant,
        };

        if let Some(name) = &arg.merchant {
            Ok(Some(Merchant::find_by_name(db, name.as_str())?))
        } else if let Some(id) = arg.merchant_id {
            Ok(Some(Merchant::find(db, (id as i64).into())?))
        } else if let Some(name) = &arg.create_merchant {
            let mut merchant = Merchant::new(name);
            merchant.save(db)?;
            Ok(Some(merchant))
        } else {
            Ok(None)
        }
    }

    pub fn category(&self, db: &Connection) -> Result<Option<Category>> {
        let arg = match self {
            RecordCommands::Add { category, .. } => category,
        };

        if let Some(name) = &arg.category {
            Ok(Some(Category::find_by_name(db, name.as_str())?))
        } else if let Some(id) = arg.category_id {
            Ok(Some(Category::find(db, (id as i64).into())?))
        } else if let Some(name) = &arg.create_category {
            let mut category = Category::new(name);
            category.save(db)?;
            Ok(Some(category))
        } else {
            Ok(None)
        }
    }
}

#[derive(Args, Clone, Debug)]
#[group(multiple = false)]
struct MerchantArgs {
    /// Name of the merchant to use
    #[arg(long, value_name = "NAME")]
    merchant: Option<String>,

    /// Id of the merchant to use
    #[arg(long, value_name = "ID")]
    merchant_id: Option<u32>,

    /// Create merchant with given name and use it
    #[arg(long, value_name = "NAME")]
    create_merchant: Option<String>,
}

#[derive(Args, Clone, Debug)]
#[group(multiple = false)]
struct CategoryArgs {
    /// Name of the category to use
    #[arg(long, value_name = "NAME")]
    category: Option<String>,

    /// Id of the category to use
    #[arg(long, value_name = "ID")]
    category_id: Option<u32>,

    /// Create category with given name and use it
    #[arg(long, value_name = "NAME")]
    create_category: Option<String>,
}

impl RecordCommands {
    pub fn operation_date(&self) -> Result<DateTime<Utc>> {
        let operation_date = match self {
            Self::Add { operation_date, .. } => operation_date,
        };

        Self::input_date_to_utc(*operation_date)
    }

    pub fn value_date(&self) -> Result<DateTime<Utc>> {
        let value_date = match self {
            Self::Add { value_date, .. } => value_date,
        };

        Self::input_date_to_utc(*value_date)
    }

    fn input_date_to_utc(date: Option<NaiveDate>) -> Result<DateTime<Utc>> {
        use chrono::{offset::MappedLocalTime, NaiveDateTime};

        match date {
            None => Ok(Utc::now()),
            Some(naive_date) => {
                match Utc.from_local_datetime(&NaiveDateTime::from(naive_date))
                {
                    MappedLocalTime::Single(date) => Ok(date),
                    MappedLocalTime::Ambiguous(date, _) => Ok(date),
                    MappedLocalTime::None => {
                        anyhow::bail!("Impossible to map local date to UTC");
                    }
                }
            }
        }
    }
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
