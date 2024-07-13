use clap::{Args, Subcommand};

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
    pub update: Option<ListUpdate>,

    /// Show only categories with this text in the name
    #[arg(long, help_heading = "Filter categories")]
    pub name: Option<String>,

    /// Maximum number of categories to show
    #[arg(short = 'c', long, help_heading = "Filter records")]
    pub count: Option<u32>,
}

#[derive(Subcommand, Clone, Debug)]
pub enum ListUpdate {
    /// Update the listed categories
    Update {},
}

#[derive(Args, Clone, Debug)]
pub struct Create {
    /// Name of the new category
    pub name: String,
}

#[derive(Args, Clone, Debug)]
pub struct Update {
    /// Name of the category to update
    pub name: String,

    /// New name of the category
    #[arg(long)]
    pub new_name: Option<String>,
}

#[derive(Args, Clone, Debug)]
pub struct Show {
    /// Name of the category to show
    pub name: String,
}

#[derive(Args, Clone, Debug)]
pub struct Delete {
    /// Name of the category to delete
    pub name: String,

    /// Confirm deletion
    #[arg(long)]
    pub confirm: bool,
}
