use crate::cli::category::{CategoryArgument, Identifier as CategoryIdentifier};
use anyhow::Result;
use clap::{Args, Subcommand};
use finnel::{merchant::NewMerchant, prelude::*};

create_identifier! {Merchant}

#[derive(Args, Clone, Debug)]
#[group(id = "merchant_args")]
pub struct MerchantArgument {
    /// Name or id of the merchant to use
    #[arg(long, value_name = "NAME_OR_ID")]
    merchant: Option<Identifier>,
}

impl MerchantArgument {
    /// Fetch the merchant selected by the user, if any
    ///
    /// Returns a Result of the eventual database operation. The first Option
    /// indicates whether or not a preference has been expressed by the user,
    /// and the second the eventual object if there is one.
    ///
    /// <no category_args> => Ok(None)
    /// --no-merchant => Ok(Some(None))
    /// --merchant 1 => Ok(Some(Some(Merchant{..})))
    pub fn resolve(
        &self,
        conn: &mut Conn,
        create: Option<&str>,
        absence: bool,
    ) -> Result<Option<Option<Merchant>>> {
        Self::resolve_with(conn, self.merchant.as_ref(), create, absence)
    }

    /// Same as the method version, but takes the identifier as parameter.
    ///
    /// This allows the definition of other struct to use the same behaviour, but with a
    /// different name
    pub fn resolve_with(
        conn: &mut Conn,
        identifier: Option<&Identifier>,
        create: Option<&str>,
        absence: bool,
    ) -> Result<Option<Option<Merchant>>> {
        if let Some(identifier) = identifier {
            Ok(Some(Some(identifier.find(conn)?)))
        } else if let Some(name) = create {
            Ok(Some(Some(NewMerchant::new(name).save(conn)?)))
        } else if absence {
            Ok(Some(None))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// List merchants
    List(List),
    /// Show details about a merchant
    Show(Show),
    /// Create a new merchant
    Create(Create),
    /// Update a merchant
    Update(Update),
    /// Delete a merchant
    Delete(Delete),
}

#[derive(Args, Clone, Debug)]
pub struct List {
    #[command(subcommand)]
    pub action: Option<Action>,

    /// Show only merchants with this text in the name
    #[arg(long, help_heading = "Filter merchants")]
    name: Option<String>,

    #[command(flatten, next_help_heading = "Filter by default category")]
    category: DefaultCategoryArgument,

    /// Show only merchants without a default category
    #[arg(
        long,
        group = "default_category_args",
        help_heading = "Filter by default category"
    )]
    no_default_category: bool,

    #[command(flatten, next_help_heading = "Filter by replacer")]
    replace_by: ReplaceByMerchantArgument,

    /// Show only merchants without a replacer
    #[arg(long, group = "replace_by_merchant_args", help_heading = "Filter by replacer")]
    no_replace_by: bool,

    /// Maximum number of merchants to show
    #[arg(short = 'c', long, help_heading = "Filter records")]
    pub count: Option<usize>,
}

impl List {
    pub fn name(&self) -> Option<String> {
        self.name.clone().map(|mut n| {
            if !n.starts_with("%") {
                n = format!("%{n}");
            }
            if !n.ends_with("%") {
                n.push('%');
            }
            n
        })
    }

    pub fn default_category(&self, conn: &mut Conn) -> Result<Option<Option<Category>>> {
        self.category.resolve(conn, None, self.no_default_category)
    }

    pub fn replace_by(&self, conn: &mut Conn) -> Result<Option<Option<Merchant>>> {
        self.replace_by.resolve(conn, None, self.no_replace_by)
    }
}

#[derive(Subcommand, Clone, Debug)]
pub enum Action {
    /// Update the listed merchant(s)
    Update(UpdateArgs),

    /// Delete the listed merchant(s)
    Delete {
        /// Confirm the deletion
        #[arg(long)]
        confirm: bool,
    },
}

#[derive(Args, Clone, Debug)]
pub struct Create {
    /// Name of the new merchant
    pub name: String,

    #[command(flatten, next_help_heading = "Category")]
    category: DefaultCategoryArgument,

    /// Create category with given name and use it
    #[arg(
        long,
        value_name = "NAME",
        group = "default_category_args",
        help_heading = "Category"
    )]
    create_default_category: Option<String>,

    #[command(flatten, next_help_heading = "Replace by")]
    replace_by: ReplaceByMerchantArgument,

    /// Create the another merchant to use instead of the currently creating
    /// one
    #[arg(
        long,
        value_name = "NAME",
        group = "replace_by_merchant_args",
        help_heading = "Replace by"
    )]
    create_replace_by: Option<String>,
}

impl Create {
    pub fn default_category(&self, conn: &mut Conn) -> Result<Option<Category>> {
        Ok(self
            .category
            .resolve(conn, self.create_default_category.as_deref(), false)?
            .flatten())
    }

    pub fn replace_by(&self, conn: &mut Conn) -> Result<Option<Merchant>> {
        Ok(self
            .replace_by
            .resolve(conn, self.create_replace_by.as_deref(), false)?
            .flatten())
    }
}

#[derive(Args, Clone, Debug)]
pub struct Update {
    #[command(flatten)]
    pub identifier: Identifier,

    #[command(flatten)]
    pub args: UpdateArgs,
}

#[derive(Args, Clone, Debug)]
pub struct UpdateArgs {
    /// New name of the merchant
    #[arg(long)]
    pub new_name: Option<String>,

    #[command(flatten, next_help_heading = "Category")]
    category: DefaultCategoryArgument,

    /// Create category with given name and use it
    #[arg(
        long,
        value_name = "NAME",
        group = "default_category_args",
        help_heading = "Category"
    )]
    create_default_category: Option<String>,

    /// Remove the category
    #[arg(long, group = "default_category_args", help_heading = "Category")]
    no_default_category: bool,

    #[command(flatten, next_help_heading = "Replace by")]
    replace_by: ReplaceByMerchantArgument,

    /// Create the another merchant to use instead of the currently creating
    /// one
    #[arg(
        long,
        value_name = "NAME",
        group = "replace_by_merchant_args",
        help_heading = "Replace by"
    )]
    create_replace_by: Option<String>,

    /// Remove the indication to replace this merchant by another one
    #[arg(long, group = "replace_by_merchant_args", help_heading = "Replace by")]
    no_replace_by: bool,
}

impl UpdateArgs {
    pub fn default_category(&self, conn: &mut Conn) -> Result<Option<Option<Category>>> {
        self.category.resolve(
            conn,
            self.create_default_category.as_deref(),
            self.no_default_category,
        )
    }

    pub fn replace_by(&self, conn: &mut Conn) -> Result<Option<Option<Merchant>>> {
        self.replace_by
            .resolve(conn, self.create_replace_by.as_deref(), self.no_replace_by)
    }
}

#[derive(Args, Clone, Debug)]
pub struct Show {
    #[command(flatten)]
    pub identifier: Identifier,

    #[command(subcommand)]
    pub action: Option<Action>,
}

#[derive(Args, Clone, Debug)]
pub struct Delete {
    #[command(flatten)]
    pub identifier: Identifier,

    /// Confirm deletion
    #[arg(long)]
    pub confirm: bool,
}

#[derive(Args, Clone, Debug)]
#[group(id = "default_category_args")]
pub struct DefaultCategoryArgument {
    /// Name or id of the category to use
    #[arg(long, value_name = "NAME_OR_ID")]
    default_category: Option<CategoryIdentifier>,
}

impl DefaultCategoryArgument {
    pub fn resolve(
        &self,
        conn: &mut Conn,
        create: Option<&str>,
        absence: bool,
    ) -> Result<Option<Option<Category>>> {
        CategoryArgument::resolve_with(conn, self.default_category.as_ref(), create, absence)
    }
}

#[derive(Args, Clone, Debug)]
#[group(id = "replace_by_merchant_args")]
pub struct ReplaceByMerchantArgument {
    /// Name or id of the merchant to use
    #[arg(long, value_name = "NAME_OR_ID")]
    replace_by: Option<Identifier>,
}

impl ReplaceByMerchantArgument {
    pub fn resolve(
        &self,
        conn: &mut Conn,
        create: Option<&str>,
        absence: bool,
    ) -> Result<Option<Option<Merchant>>> {
        MerchantArgument::resolve_with(conn, self.replace_by.as_ref(), create, absence)
    }
}
