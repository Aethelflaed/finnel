use anyhow::Result;

use clap::{Args, Subcommand};

use finnel::{category::NewCategory, prelude::*};

create_identifier! {Category}

#[derive(Args, Clone, Debug)]
#[group(id = "category_args")]
pub struct CategoryArgument {
    /// Name or id of the category to use
    #[arg(long, value_name = "NAME_OR_ID")]
    category: Option<Identifier>,
}

impl CategoryArgument {
    /// Fetch the category selected by the user, if any
    ///
    /// Returns a Result of the eventual database operation. The first Option
    /// indicates whether or not a preference has been expressed by the user,
    /// and the second the eventual object if there is one.
    ///
    /// <no category_args> => Ok(None)
    /// --no-category => Ok(Some(None))
    /// --category 1 => Ok(Some(Some(Category{..})))
    pub fn resolve(
        &self,
        conn: &mut Conn,
        create: Option<&str>,
        absence: bool,
    ) -> Result<Option<Option<Category>>> {
        Self::resolve_with(conn, self.category.as_ref(), create, absence)
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
    ) -> Result<Option<Option<Category>>> {
        if let Some(identifier) = identifier {
            Ok(Some(Some(identifier.find(conn)?)))
        } else if let Some(name) = create {
            Ok(Some(Some(NewCategory::new(name).save(conn)?)))
        } else if absence {
            Ok(Some(None))
        } else {
            Ok(None)
        }
    }
}

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

    #[command(flatten, next_help_heading = "Filter by parent")]
    parent: ParentCategoryArgument,

    /// Show only categories without a parent
    #[arg(long, group = "parent_args", help_heading = "Filter by parent")]
    no_parent: bool,

    #[command(flatten, next_help_heading = "Filter by replacer")]
    replace_by: ReplaceByCategoryArgument,

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

    #[command(flatten, next_help_heading = "Parent")]
    parent: ParentCategoryArgument,

    /// Create the another category to use as the parent of the currently
    /// creating one
    #[arg(
        long,
        value_name = "NAME",
        group = "parent_args",
        help_heading = "Parent"
    )]
    create_parent: Option<String>,

    #[command(flatten, next_help_heading = "Replace by")]
    replace_by: ReplaceByCategoryArgument,

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

    #[command(flatten, next_help_heading = "Parent")]
    parent: ParentCategoryArgument,

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

    #[command(flatten, next_help_heading = "Replace by")]
    replace_by: ReplaceByCategoryArgument,

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
#[group(id = "parent_args")]
pub struct ParentCategoryArgument {
    /// Name or id of the category to use
    #[arg(long, value_name = "NAME_OR_ID")]
    parent: Option<Identifier>,
}

impl ParentCategoryArgument {
    pub fn resolve(
        &self,
        conn: &mut Conn,
        create: Option<&str>,
        absence: bool,
    ) -> Result<Option<Option<Category>>> {
        CategoryArgument::resolve_with(conn, self.parent.as_ref(), create, absence)
    }
}

#[derive(Args, Clone, Debug)]
#[group(id = "replace_by_args")]
pub struct ReplaceByCategoryArgument {
    /// Name or id of the category to use
    #[arg(long, value_name = "NAME_OR_ID")]
    replace_by: Option<Identifier>,
}

impl ReplaceByCategoryArgument {
    pub fn resolve(
        &self,
        conn: &mut Conn,
        create: Option<&str>,
        absence: bool,
    ) -> Result<Option<Option<Category>>> {
        CategoryArgument::resolve_with(conn, self.replace_by.as_ref(), create, absence)
    }
}
