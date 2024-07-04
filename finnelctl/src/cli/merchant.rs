use clap::{Args, Subcommand};

use finnel::Id;

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// List merchants
    List(List),
    /// Create a new merchant
    Create(Create),
    /// Update a merchant
    Update(Update),
    /// Show details about a merchant
    Show(Show),
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
}

#[derive(Args, Clone, Debug)]
pub struct Update {
    /// Id of the merchant to update
    id: u32,

    /// New name of the merchant
    #[arg(long)]
    pub name: Option<String>,
}

impl Update {
    pub fn id(&self) -> Id {
        (self.id as i64).into()
    }
}

#[derive(Args, Clone, Debug)]
pub struct Show {
    /// Id of the merchant to show
    id: u32,
}

impl Show {
    pub fn id(&self) -> Id {
        (self.id as i64).into()
    }
}

#[derive(Args, Clone, Debug)]
pub struct Delete {
    /// Id of the merchant to delete
    id: u32,

    /// Confirm deletion
    #[arg(long)]
    pub confirm: bool,
}

impl Delete {
    pub fn id(&self) -> Id {
        (self.id as i64).into()
    }
}
