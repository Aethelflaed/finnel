use anyhow::Result;

use clap::{Args, Subcommand};

use finnel::{category::NewCategory, prelude::*};

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// List categories
    List(List),
    /// Show details about a category
    Show(Show),
    /// Create a new category
    Create(Create),
    /// Update a category
    Update(Update),
    /// Delete a category
    Delete(Delete),
}

#[derive(Args, Clone, Debug)]
pub struct List {
    #[command(subcommand)]
    pub action: Option<Action>,

    /// Show only categories with this text in the name
    #[arg(long, help_heading = "Filter categories")]
    name: Option<String>,

    #[allow(private_interfaces)]
    #[command(flatten, next_help_heading = "Filter by parent")]
    parent: ParentArgs,

    /// Show only categories without a parent
    #[arg(long, group = "parent_args", help_heading = "Filter by parent")]
    no_parent: bool,

    #[allow(private_interfaces)]
    #[command(flatten, next_help_heading = "Filter by replacer")]
    replace_by: ReplaceByArgs,

    /// Show only categories without a replacer
    #[arg(long, group = "replace_by_args", help_heading = "Filter by replacer")]
    no_replace_by: bool,

    /// Maximum number of categories to show
    #[arg(short = 'c', long, help_heading = "Filter records")]
    pub count: Option<u32>,
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

    pub fn parent(&self, conn: &mut Conn) -> Result<Option<Option<Category>>> {
        self.parent.resolve(conn, None, self.no_parent)
    }

    pub fn replace_by(&self, conn: &mut Conn) -> Result<Option<Option<Category>>> {
        self.replace_by.resolve(conn, None, self.no_replace_by)
    }
}

#[derive(Subcommand, Clone, Debug)]
pub enum Action {
    /// Update the listed category(ies)
    Update(UpdateArgs),

    /// Delete the listed category(ies)
    Delete {
        /// Confirm the deletion
        #[arg(long)]
        confirm: bool,
    },
}

#[derive(Args, Clone, Debug)]
pub struct Create {
    /// Name of the new category
    pub name: String,

    #[allow(private_interfaces)]
    #[command(flatten, next_help_heading = "Parent")]
    parent: ParentArgs,

    /// Create the another category to use as the parent of the currently
    /// creating one
    #[arg(
        long,
        value_name = "NAME",
        group = "parent_args",
        help_heading = "Parent"
    )]
    create_parent: Option<String>,

    #[allow(private_interfaces)]
    #[command(flatten, next_help_heading = "Replace by")]
    replace_by: ReplaceByArgs,

    /// Create the another category to use instead of the currently creating
    /// one
    #[arg(
        long,
        value_name = "NAME",
        group = "replace_by_args",
        help_heading = "Replace by"
    )]
    create_replace_by: Option<String>,
}

impl Create {
    pub fn parent(&self, conn: &mut Conn) -> Result<Option<Category>> {
        Ok(self
            .parent
            .resolve(conn, self.create_parent.as_deref(), false)?
            .flatten())
    }

    pub fn replace_by(&self, conn: &mut Conn) -> Result<Option<Category>> {
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
    /// New name of the category
    #[arg(long)]
    pub new_name: Option<String>,

    #[allow(private_interfaces)]
    #[command(flatten, next_help_heading = "Parent")]
    parent: ParentArgs,

    /// Create the another category to use as the parent of the currently
    /// creating one
    #[arg(
        long,
        value_name = "NAME",
        group = "parent_args",
        help_heading = "Parent"
    )]
    create_parent: Option<String>,

    /// Remove the relation to the parent category
    #[arg(long, group = "parent_args", help_heading = "Parent")]
    no_parent: bool,

    #[allow(private_interfaces)]
    #[command(flatten, next_help_heading = "Replace by")]
    replace_by: ReplaceByArgs,

    /// Create the another category to use instead of the currently creating
    /// one
    #[arg(
        long,
        value_name = "NAME",
        group = "replace_by_args",
        help_heading = "Replace by"
    )]
    create_replace_by: Option<String>,

    /// Remove the indication to replace this category by another one
    #[arg(long, group = "replace_by_args", help_heading = "Replace by")]
    no_replace_by: bool,
}

impl UpdateArgs {
    pub fn parent(&self, conn: &mut Conn) -> Result<Option<Option<Category>>> {
        self.parent
            .resolve(conn, self.create_parent.as_deref(), self.no_parent)
    }

    pub fn replace_by(&self, conn: &mut Conn) -> Result<Option<Option<Category>>> {
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
#[group(id = "parent_args", multiple = false)]
struct ParentArgs {
    /// Name of the category to use as the parent of the current one
    #[arg(long, value_name = "NAME")]
    parent: Option<String>,

    /// Id of the category to use as the parent of the current one
    #[arg(long, value_name = "ID")]
    parent_id: Option<u32>,
}

impl ParentArgs {
    /// Fetch the category selected by the user, if any
    ///
    /// Returns a Result of the eventual database operation. The first Option
    /// indicates whether or not a preference has been expressed by the user,
    /// and the second the eventual object if there is one.
    ///
    /// <no parent_args> => Ok(None)
    /// --no-parent => Ok(Some(None))
    /// --parent-id 1 => Ok(Some(Some(Category{..})))
    pub fn resolve(
        &self,
        conn: &mut Conn,
        create: Option<&str>,
        absence: bool,
    ) -> Result<Option<Option<Category>>> {
        if let Some(name) = &self.parent {
            Ok(Some(Some(Category::find_by_name(conn, name.as_str())?)))
        } else if let Some(id) = self.parent_id {
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
#[group(id = "replace_by_args", multiple = false)]
struct ReplaceByArgs {
    /// Name of the category to replace the current one
    #[arg(long, value_name = "NAME")]
    replace_by: Option<String>,

    /// Id of the category to replace the current one
    #[arg(long, value_name = "ID")]
    replace_by_id: Option<u32>,
}

impl ReplaceByArgs {
    /// Fetch the category selected by the user, if any
    ///
    /// Returns a Result of the eventual database operation. The first Option
    /// indicates whether or not a preference has been expressed by the user,
    /// and the second the eventual object if there is one.
    ///
    /// <no replace_by_args> => Ok(None)
    /// --no-replace-by => Ok(Some(None))
    /// --replace-by-id 1 => Ok(Some(Some(Category{..})))
    pub fn resolve(
        &self,
        conn: &mut Conn,
        create: Option<&str>,
        absence: bool,
    ) -> Result<Option<Option<Category>>> {
        if let Some(name) = &self.replace_by {
            Ok(Some(Some(Category::find_by_name(conn, name.as_str())?)))
        } else if let Some(id) = self.replace_by_id {
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
pub struct Identifier {
    /// Name or id of the category
    pub name_or_id: String,
}

impl Identifier {
    pub fn find(&self, conn: &mut Conn) -> Result<Category> {
        if self.name_or_id.chars().all(|c| c.is_ascii_digit()) {
            Ok(Category::find(conn, self.name_or_id.parse()?)?)
        } else {
            Ok(Category::find_by_name(conn, &self.name_or_id)?)
        }
    }
}

impl From<String> for Identifier {
    fn from(value: String) -> Self {
        Self { name_or_id: value }
    }
}
