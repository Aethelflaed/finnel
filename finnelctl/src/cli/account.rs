use clap::{Args, Subcommand};

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// List registered accounts
    List(List),
    /// Show details about an account
    Show(Show),
    /// Create a new account
    Create(Create),
    /// Delete an account
    Delete(Delete),
    /// Check or set the default account
    Default(Default),
}

#[derive(Args, Clone, Debug)]
pub struct List {}

#[derive(Args, Clone, Debug)]
pub struct Create {
    /// Name of the new category
    pub name: String,
}

#[derive(Args, Clone, Debug)]
pub struct Update {
    /// Name of the category to update
    pub name: Option<String>,

    /// New name of the category
    #[arg(long)]
    pub new_name: Option<String>,
}

#[derive(Args, Clone, Debug)]
pub struct Show {
    /// Name of the category to show
    pub name: Option<String>,
}

#[derive(Args, Clone, Debug)]
pub struct Delete {
    /// Name of the category to delete
    pub name: Option<String>,

    /// Confirm deletion
    #[arg(long)]
    pub confirm: bool,
}

#[derive(Args, Clone, Debug)]
pub struct Default {
    /// Name of the category to delete
    pub name: Option<String>,

    /// Reset the default account
    #[arg(short, long)]
    #[arg(long)]
    pub reset: bool,
}
