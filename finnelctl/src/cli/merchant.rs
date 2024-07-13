use anyhow::Result;

use clap::{Args, Subcommand};

use finnel::{category::NewCategory, prelude::*};

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
    pub update: Option<ListUpdate>,

    /// Show only merchants with this text in the name
    #[arg(long, help_heading = "Filter merchants")]
    pub name: Option<String>,

    /// Maximum number of merchants to show
    #[arg(short = 'c', long, help_heading = "Filter records")]
    pub count: Option<usize>,
}

#[derive(Subcommand, Clone, Debug)]
pub enum ListUpdate {
    /// Update the listed merchants
    Update {},
}

#[derive(Args, Clone, Debug)]
pub struct Create {
    /// Name of the new merchant
    pub name: String,

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
    create_default_category: Option<String>,
}

impl Create {
    pub fn default_category(
        &self,
        conn: &mut Conn,
    ) -> Result<Option<Option<Category>>> {
        self.category
            .resolve(conn, self.create_default_category.clone(), false)
    }
}

#[derive(Args, Clone, Debug)]
pub struct Update {
    /// Name of the merchant to update
    pub name: String,

    /// New name of the merchant
    #[arg(long)]
    pub new_name: Option<String>,

    #[allow(private_interfaces)]
    #[command(flatten, next_help_heading = "Category")]
    category: CategoryArgs,

    /// Remove the category
    #[arg(long, group = "category_args", help_heading = "Category")]
    no_default_category: bool,
}

impl Update {
    pub fn default_category(
        &self,
        conn: &mut Conn,
    ) -> Result<Option<Option<Category>>> {
        self.category.resolve(conn, None, self.no_default_category)
    }
}

#[derive(Args, Clone, Debug)]
pub struct Show {
    /// Name of the merchant to show
    pub name: String,
}

#[derive(Args, Clone, Debug)]
pub struct Delete {
    /// Name of the merchant to delete
    pub name: String,

    /// Confirm deletion
    #[arg(long)]
    pub confirm: bool,
}

#[derive(Args, Clone, Debug)]
#[group(id = "category_args", multiple = false)]
struct CategoryArgs {
    /// Name of the category to use
    #[arg(long, value_name = "NAME")]
    default_category: Option<String>,

    /// Id of the category to use
    #[arg(long, value_name = "ID")]
    default_category_id: Option<u32>,
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
        create: Option<String>,
        absence: bool,
    ) -> Result<Option<Option<Category>>> {
        if let Some(name) = &self.default_category {
            Ok(Some(Some(Category::find_by_name(conn, name.as_str())?)))
        } else if let Some(id) = self.default_category_id {
            Ok(Some(Some(Category::find(conn, id as i64)?)))
        } else if let Some(name) = create {
            Ok(Some(Some(NewCategory::new(&name).save(conn)?)))
        } else if absence {
            Ok(Some(None))
        } else {
            Ok(None)
        }
    }
}
