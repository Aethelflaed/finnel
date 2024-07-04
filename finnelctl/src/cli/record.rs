use std::path::PathBuf;

use crate::utils::naive_date_to_utc;

use finnel::{
    transaction::{Direction, Mode},
    Category, Connection, Decimal, Entity, Merchant,
};

use anyhow::Result;
use clap::{Args, Subcommand};
use chrono::{offset::Utc, DateTime, NaiveDate};

#[derive(Debug, Clone, Subcommand)]
pub enum RecordCommands {
    /// Add a new record
    Add {
        /// Amount of the record
        ///
        /// Without currency symbol, the currency is inferred from the account
        amount: Decimal,

        /// Describe the record
        details: String,

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
        #[command(flatten, next_help_heading = "Category")]
        category: CategoryArgs,

        /// Create category with given name and use it
        #[arg(
            long,
            value_name = "NAME",
            group = "category_args",
            help_heading = "Category"
        )]
        create_category: Option<String>,

        #[allow(private_interfaces)]
        #[command(flatten, next_help_heading = "Merchant")]
        merchant: MerchantArgs,

        /// Create merchant with given name and use it
        #[arg(
            long,
            value_name = "NAME",
            group = "merchant_args",
            help_heading = "Merchant"
        )]
        create_merchant: Option<String>,
    },
    /// List records
    List {
        #[command(subcommand)]
        set: Option<RecordListSetCommand>,

        /// Show only records from after this date
        #[arg(
            short = 'a',
            long,
            value_name = "DATE",
            help_heading = "Filter records"
        )]
        after: Option<NaiveDate>,

        /// Show only records from before this date
        #[arg(
            short = 'b',
            long,
            value_name = "DATE",
            help_heading = "Filter records"
        )]
        before: Option<NaiveDate>,

        /// Sort and filter according to the operation date instead of the
        /// value date
        #[arg(short = 'o', long, help_heading = "Filter records")]
        operation_date: bool,

        /// Show only records with an amount greater than this one
        #[arg(
            short = 'g',
            long,
            alias = "gt",
            value_name = "AMOUNT",
            help_heading = "Filter records"
        )]
        greater_than: Option<Decimal>,

        /// Show only records with an amount less than this one
        #[arg(
            short = 'l',
            long,
            alias = "lt",
            value_name = "AMOUNT",
            help_heading = "Filter records"
        )]
        less_than: Option<Decimal>,

        /// Transaction direction
        #[arg(short = 'd', long, help_heading = "Filter records")]
        direction: Option<Direction>,

        /// Transaction mode
        #[arg(short = 'm', long, help_heading = "Filter records")]
        mode: Option<Mode>,

        /// Show only records with this text in the details
        #[arg(long, help_heading = "Filter records")]
        details: Option<String>,

        /// Maximum number of records to show
        #[arg(short = 'c', long, help_heading = "Filter records")]
        count: Option<usize>,

        #[allow(private_interfaces)]
        #[command(flatten, next_help_heading = "Filter by category")]
        category: CategoryArgs,

        /// Show only records without a category
        #[arg(
            long,
            group = "category_args",
            help_heading = "Filter by category"
        )]
        no_category: bool,

        #[allow(private_interfaces)]
        #[command(flatten, next_help_heading = "Filter by merchant")]
        merchant: MerchantArgs,

        /// Show only records without a merchant
        #[arg(
            long,
            group = "merchant_args",
            help_heading = "Filter by merchant"
        )]
        no_merchant: bool,
    },
    /// Import records from a transaction CSV file
    Import {
        /// File to import
        file: PathBuf,

        /// Import profile to use
        #[arg(short = 'P', long, help_heading = "Import records")]
        profile: String,
    },
}

#[derive(Subcommand, Clone, Debug)]
pub enum RecordListSetCommand {
    /// Update the listed records
    Set(RecordUpdateArgs),
}

#[derive(Args, Clone, Debug)]
pub struct RecordUpdateArgs {
    /// Change the record details
    details: Option<String>,

    /// Change the value date
    #[arg(long, value_name = "DATE", help_heading = "Record")]
    value_date: Option<NaiveDate>,

    #[allow(private_interfaces)]
    #[command(flatten, next_help_heading = "Category")]
    category: CategoryArgs,

    /// Create category with given name and use it
    #[arg(
        long,
        value_name = "NAME",
        group = "category_args",
        help_heading = "Category"
    )]
    create_category: Option<String>,

    /// Remove the category
    #[arg(
        long,
        group = "category_args",
        help_heading = "Category"
    )]
    no_category: bool,

    #[allow(private_interfaces)]
    #[command(flatten, next_help_heading = "Merchant")]
    merchant: MerchantArgs,

    /// Create merchant with given name and use it
    #[arg(
        long,
        value_name = "NAME",
        group = "merchant_args",
        help_heading = "Merchant"
    )]
    create_merchant: Option<String>,

    /// Remove the merchant
    #[arg(
        long,
        group = "merchant_args",
        help_heading = "Merchant"
    )]
    no_merchant: bool,
}

#[derive(Args, Clone, Debug)]
#[group(id = "category_args", multiple = false)]
struct CategoryArgs {
    /// Name of the category to use
    #[arg(long, value_name = "NAME")]
    category: Option<String>,

    /// Id of the category to use
    #[arg(long, value_name = "ID")]
    category_id: Option<u32>,
}

#[derive(Args, Clone, Debug)]
#[group(id = "merchant_args", multiple = false)]
struct MerchantArgs {
    /// Name of the merchant to use
    #[arg(long, value_name = "NAME")]
    merchant: Option<String>,

    /// Id of the merchant to use
    #[arg(long, value_name = "ID")]
    merchant_id: Option<u32>,
}

impl RecordCommands {
    /// Fetch the category selected by the user, if any
    ///
    /// Returns a Result of the eventual database operation. The first Option
    /// indicates whether or not a preference has been expressed by the user,
    /// and the second the eventual object if there is one.
    ///
    /// <no category_args> => Ok(None)
    /// --no-category => Ok(Some(None))
    /// --category-id 1 => Ok(Some(Some(Category{..})))
    pub fn category(
        &self,
        db: &Connection,
    ) -> Result<Option<Option<Category>>> {
        let (arg, create, no) = match self {
            Self::Add {
                category,
                create_category,
                ..
            } => (category, create_category, false),
            Self::List {
                category,
                no_category,
                ..
            } => (category, &None, *no_category),
            Self::Import { .. } => {
                anyhow::bail!("Not defined on this variant")
            }
        };

        if let Some(name) = &arg.category {
            Ok(Some(Some(Category::find_by_name(db, name.as_str())?)))
        } else if let Some(id) = arg.category_id {
            Ok(Some(Some(Category::find(db, (id as i64).into())?)))
        } else if let Some(name) = create {
            let mut category = Category::new(name);
            category.save(db)?;
            Ok(Some(Some(category)))
        } else if no {
            Ok(Some(None))
        } else {
            Ok(None)
        }
    }

    /// Fetch the merchant selected by the user, if any
    ///
    /// Returns a Result of the eventual database operation. The first Option
    /// indicates whether or not a preference has been expressed by the user,
    /// and the second the eventual object if there is one.
    ///
    /// <no category_args> => Ok(None)
    /// --no-merchant => Ok(Some(None))
    /// --merchant-id 1 => Ok(Some(Some(Merchant{..})))
    pub fn merchant(
        &self,
        db: &Connection,
    ) -> Result<Option<Option<Merchant>>> {
        let (arg, create, no) = match self {
            Self::Add {
                merchant,
                create_merchant,
                ..
            } => (merchant, create_merchant, false),
            Self::List {
                merchant,
                no_merchant,
                ..
            } => (merchant, &None, *no_merchant),
            Self::Import { .. } => {
                anyhow::bail!("Not defined on this variant")
            }
        };

        if let Some(name) = &arg.merchant {
            Ok(Some(Some(Merchant::find_by_name(db, name.as_str())?)))
        } else if let Some(id) = arg.merchant_id {
            Ok(Some(Some(Merchant::find(db, (id as i64).into())?)))
        } else if let Some(name) = create {
            let mut merchant = Merchant::new(name);
            merchant.save(db)?;
            Ok(Some(Some(merchant)))
        } else if no {
            Ok(Some(None))
        } else {
            Ok(None)
        }
    }

    pub fn operation_date(&self) -> Result<DateTime<Utc>> {
        let date = match self {
            Self::Add { operation_date, .. } => operation_date,
            Self::List { .. } | Self::Import { .. } => {
                anyhow::bail!("Not defined on this variant")
            }
        };

        date.map(naive_date_to_utc).unwrap_or(Ok(Utc::now()))
    }

    pub fn value_date(&self) -> Result<DateTime<Utc>> {
        let date = match self {
            Self::Add { value_date, .. } => value_date,
            Self::List { .. } | Self::Import { .. } => {
                anyhow::bail!("Not defined on this variant")
            }
        };

        date.map(naive_date_to_utc).unwrap_or(Ok(Utc::now()))
    }

    pub fn after(&self) -> Result<Option<DateTime<Utc>>> {
        let date = match self {
            Self::List { after, .. } => after,
            Self::Add { .. } | Self::Import { .. } => {
                anyhow::bail!("Not defined on this variant")
            }
        };

        date.map(naive_date_to_utc).transpose()
    }

    pub fn before(&self) -> Result<Option<DateTime<Utc>>> {
        let date = match self {
            Self::List { before, .. } => before,
            Self::Add { .. } | Self::Import { .. } => {
                anyhow::bail!("Not defined on this variant")
            }
        };

        date.map(naive_date_to_utc).transpose()
    }
}
