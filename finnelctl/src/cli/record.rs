use finnel::{category::NewCategory, merchant::NewMerchant, prelude::*};

use anyhow::Result;
use chrono::{NaiveDate, Utc};
use clap::{builder::PossibleValue, Args, Subcommand, ValueEnum};

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// List records
    List(List),
    /// Show details about a record
    Show(Show),
    /// Create a new record
    Create(Create),
    /// Update a record
    Update(Update),
}

#[derive(Args, Clone, Debug)]
pub struct Show {
    #[command(subcommand)]
    pub action: Option<ShowAction>,

    /// Id of the record to show
    id: u32,
}

impl Show {
    pub fn id(&self) -> i64 {
        self.id as i64
    }
}

#[derive(Subcommand, Clone, Debug)]
pub enum ShowAction {
    /// Split the record
    Split(Split),
    #[command(flatten)]
    Other(Action),
}

#[derive(Args, Clone, Debug)]
pub struct Split {
    /// Amount of the record to split into a new record
    #[arg(help_heading = "New record")]
    pub amount: Decimal,

    #[arg(long, help_heading = "New record")]
    pub details: Option<String>,

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

    /// Assign no category to the new record
    #[arg(long, group = "category_args", help_heading = "Category")]
    no_category: bool,
}

impl Split {
    pub fn category(&self, conn: &mut Conn) -> Result<Option<Option<Category>>> {
        self.category
            .resolve(conn, self.create_category.as_deref(), self.no_category)
    }
}

#[derive(Args, Clone, Debug)]
pub struct Create {
    /// Amount of the record
    ///
    /// Without currency symbol, the currency is inferred from the account
    #[arg(help_heading = "Record")]
    pub amount: Decimal,

    /// Describe the record
    #[arg(help_heading = "Record")]
    pub details: String,

    /// Transaction direction
    ///
    /// Possible values include debit, credit, and variants
    #[arg(short = 'd', long, default_value_t, help_heading = "Record")]
    pub direction: Direction,

    /// Transaction mode
    ///
    /// Possible values include direct, transfer, ATM, ATM CB *WXYZ, CB *WXYZ
    #[arg(short = 'm', long, default_value_t, help_heading = "Record")]
    pub mode: Mode,

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
}

impl Create {
    pub fn operation_date(&self) -> NaiveDate {
        self.operation_date
            .unwrap_or_else(|| Utc::now().date_naive())
    }

    pub fn value_date(&self) -> NaiveDate {
        self.value_date.unwrap_or_else(|| Utc::now().date_naive())
    }

    pub fn category(&self, conn: &mut Conn) -> Result<Option<Category>> {
        Ok(self
            .category
            .resolve(conn, self.create_category.as_deref(), false)?
            .flatten())
    }

    pub fn merchant(&self, conn: &mut Conn) -> Result<Option<Merchant>> {
        Ok(self
            .merchant
            .resolve(conn, self.create_merchant.as_deref(), false)?
            .flatten())
    }
}

#[derive(Args, Clone, Debug)]
pub struct Update {
    /// Id of the record to update
    id: u32,

    #[command(flatten)]
    pub args: UpdateArgs,
}

impl Update {
    pub fn id(&self) -> i64 {
        self.id as i64
    }
}

use finnel::record::query::{OrderDirection, OrderField};

#[derive(Debug, Clone, Copy, derive_more::Into)]
pub struct Sort(OrderField, OrderDirection);

impl Sort {
    pub fn try_from(value: &str) -> Result<Self> {
        Self::from_str(value, true).map_err(|e| anyhow::anyhow!("Cannot construct sort with {}", e))
    }
}

impl ValueEnum for Sort {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            Sort(OrderField::Amount, OrderDirection::Asc),
            Sort(OrderField::Date, OrderDirection::Asc),
            Sort(OrderField::CategoryId, OrderDirection::Asc),
            Sort(OrderField::MerchantId, OrderDirection::Asc),
            Sort(OrderField::Amount, OrderDirection::Desc),
            Sort(OrderField::Date, OrderDirection::Desc),
            Sort(OrderField::CategoryId, OrderDirection::Desc),
            Sort(OrderField::MerchantId, OrderDirection::Desc),
        ]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        let mut value = PossibleValue::new(self.to_string());

        if let OrderField::Date = self.0 {
            value = value.help("Value or operation date, depending on --operation-date");
        }

        Some(value)
    }
}

impl core::fmt::Display for Sort {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.0 {
            OrderField::Amount => match self.1 {
                OrderDirection::Asc => write!(f, "amount"),
                OrderDirection::Desc => write!(f, "amount.desc"),
            },
            OrderField::Date => match self.1 {
                OrderDirection::Asc => write!(f, "date"),
                OrderDirection::Desc => write!(f, "date.desc"),
            },
            OrderField::CategoryId => match self.1 {
                OrderDirection::Asc => write!(f, "category_id"),
                OrderDirection::Desc => write!(f, "category_id.desc"),
            },
            OrderField::MerchantId => match self.1 {
                OrderDirection::Asc => write!(f, "merchant_id"),
                OrderDirection::Desc => write!(f, "merchant_id.desc"),
            },
        }
    }
}

#[derive(Args, Clone, Debug)]
pub struct List {
    #[command(subcommand)]
    pub action: Option<Action>,

    /// Show only records from after this date
    #[arg(
        short = 'a',
        long,
        value_name = "DATE",
        help_heading = "Filter records"
    )]
    pub from: Option<NaiveDate>,

    /// Show only records from before this date
    #[arg(
        short = 'b',
        long,
        value_name = "DATE",
        help_heading = "Filter records"
    )]
    pub to: Option<NaiveDate>,

    /// Sort and filter according to the operation date instead of the
    /// value date
    #[arg(short = 'o', long, help_heading = "Filter records")]
    pub operation_date: bool,

    /// Show only records with an amount greater than this one
    #[arg(
        short = 'g',
        long,
        alias = "gt",
        value_name = "AMOUNT",
        help_heading = "Filter records"
    )]
    pub greater_than: Option<Decimal>,

    /// Show only records with an amount less than this one
    #[arg(
        short = 'l',
        long,
        alias = "lt",
        value_name = "AMOUNT",
        help_heading = "Filter records"
    )]
    pub less_than: Option<Decimal>,

    /// Transaction direction
    #[arg(short = 'd', long, help_heading = "Filter records")]
    pub direction: Option<Direction>,

    /// Transaction mode
    #[arg(short = 'm', long, help_heading = "Filter records")]
    pub mode: Option<Mode>,

    /// Show only records with this text in the details
    #[arg(long, help_heading = "Filter records")]
    details: Option<String>,

    /// Maximum number of records to show
    #[arg(short = 'c', long, help_heading = "Filter records")]
    pub count: Option<i64>,

    #[arg(long, help_heading = "Sort records")]
    pub sort: Vec<Sort>,

    #[allow(private_interfaces)]
    #[command(flatten, next_help_heading = "Filter by category")]
    category: CategoryArgs,

    /// Show only records without a category
    #[arg(long, group = "category_args", help_heading = "Filter by category")]
    no_category: bool,

    #[allow(private_interfaces)]
    #[command(flatten, next_help_heading = "Filter by merchant")]
    merchant: MerchantArgs,

    /// Show only records without a merchant
    #[arg(long, group = "merchant_args", help_heading = "Filter by merchant")]
    no_merchant: bool,
}

impl List {
    pub fn details(&self) -> Option<String> {
        self.details.clone().map(|mut n| {
            if !n.starts_with("%") {
                n = format!("%{n}");
            }
            if !n.ends_with("%") {
                n.push('%');
            }
            n
        })
    }

    pub fn category(&self, conn: &mut Conn) -> Result<Option<Option<Category>>> {
        self.category.resolve(conn, None, self.no_category)
    }

    pub fn merchant(&self, conn: &mut Conn) -> Result<Option<Option<Merchant>>> {
        self.merchant.resolve(conn, None, self.no_merchant)
    }
}

#[derive(Subcommand, Clone, Debug)]
pub enum Action {
    /// Update the listed record(s)
    Update(UpdateArgs),

    /// Delete the listed record(s)
    Delete {
        /// Confirm the deletion
        #[arg(long)]
        confirm: bool,
    },
}

#[derive(Args, Clone, Debug)]
pub struct UpdateArgs {
    /// Change the record details
    #[arg(long, value_name = "DETAILS", help_heading = "Record")]
    pub details: Option<String>,

    /// Change the value date
    #[arg(long, value_name = "DATE", help_heading = "Record")]
    pub value_date: Option<NaiveDate>,

    /// Confirm update of sensitive information
    #[arg(long)]
    pub confirm: bool,

    /// Amount of the record
    #[arg(long, requires = "confirm", help_heading = "Record")]
    pub amount: Option<Decimal>,

    /// Transaction direction
    ///
    /// Possible values include debit, credit, and variants
    #[arg(short = 'd', long, requires = "confirm", help_heading = "Record")]
    pub direction: Option<Direction>,

    /// Transaction mode
    ///
    /// Possible values include direct, transfer, ATM, ATM CB *WXYZ, CB *WXYZ
    #[arg(short = 'm', long, requires = "confirm", help_heading = "Record")]
    pub mode: Option<Mode>,

    /// Operation date
    #[arg(
        long,
        value_name = "DATE",
        requires = "confirm",
        help_heading = "Record"
    )]
    pub operation_date: Option<NaiveDate>,

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
    #[arg(long, group = "category_args", help_heading = "Category")]
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
    #[arg(long, group = "merchant_args", help_heading = "Merchant")]
    no_merchant: bool,
}

impl UpdateArgs {
    pub fn category(&self, conn: &mut Conn) -> Result<Option<Option<Category>>> {
        self.category
            .resolve(conn, self.create_category.as_deref(), self.no_category)
    }

    pub fn merchant(&self, conn: &mut Conn) -> Result<Option<Option<Merchant>>> {
        self.merchant
            .resolve(conn, self.create_merchant.as_deref(), self.no_merchant)
    }
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

impl CategoryArgs {
    /// Fetch the category selected by the user, if any
    ///
    /// Returns a Result of the eventual database operation. The first Option
    /// indicates whether or not a preference has been expressed by the user,
    /// and the second the eventual object if there is one.
    ///
    /// <no category_args> => Ok(None)
    /// --no-category => Ok(Some(None))
    /// --category-id 1 => Ok(Some(Some(Category{..})))
    pub fn resolve(
        &self,
        conn: &mut Conn,
        create: Option<&str>,
        absence: bool,
    ) -> Result<Option<Option<Category>>> {
        if let Some(name) = &self.category {
            Ok(Some(Some(Category::find_by_name(conn, name.as_str())?)))
        } else if let Some(id) = self.category_id {
            Ok(Some(Some(Category::find(conn, id as i64)?)))
        } else if let Some(name) = create {
            Ok(Some(Some(NewCategory::new(name).save(conn)?)))
        } else if absence {
            Ok(Some(None))
        } else {
            Ok(None)
        }
    }
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

impl MerchantArgs {
    /// Fetch the merchant selected by the user, if any
    ///
    /// Returns a Result of the eventual database operation. The first Option
    /// indicates whether or not a preference has been expressed by the user,
    /// and the second the eventual object if there is one.
    ///
    /// <no merchant_args> => Ok(None)
    /// --no-merchant => Ok(Some(None))
    /// --merchant-id 1 => Ok(Some(Some(Merchant{..})))
    pub fn resolve(
        &self,
        conn: &mut Conn,
        create: Option<&str>,
        absence: bool,
    ) -> Result<Option<Option<Merchant>>> {
        if let Some(name) = &self.merchant {
            Ok(Some(Some(Merchant::find_by_name(conn, name.as_str())?)))
        } else if let Some(id) = self.merchant_id {
            Ok(Some(Some(Merchant::find(conn, id as i64)?)))
        } else if let Some(name) = create {
            Ok(Some(Some(NewMerchant::new(name).save(conn)?)))
        } else if absence {
            Ok(Some(None))
        } else {
            Ok(None)
        }
    }
}
