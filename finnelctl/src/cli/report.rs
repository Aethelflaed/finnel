use anyhow::Result;

use clap::{Args, Subcommand};

use crate::cli::category::Identifier as CategoryIdentifier;
use finnel::prelude::*;

create_identifier! {Report}

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// List reports
    List(List),
    /// Show a specific report
    Show(Show),
    /// Create a report
    Create(Create),
    /// Delete a report
    Delete(Delete),
}

#[derive(Args, Clone, Debug)]
pub struct List {}

#[derive(Args, Clone, Debug)]
pub struct Show {
    #[command(flatten)]
    pub identifier: Identifier,

    #[command(subcommand)]
    pub action: Option<Action>,
}

#[derive(Subcommand, Clone, Debug)]
pub enum Action {
    /// Add categories to the report
    Add {
        /// Name or id (or a mix of both) of categories to add
        categories: Vec<CategoryIdentifier>,
    },
    /// Remove categories from the report
    Remove {
        /// Name or id (or a mix of both) of categories to add
        categories: Vec<CategoryIdentifier>,
    },
}

#[derive(Args, Clone, Debug)]
pub struct Create {
    /// Name of the new report
    pub name: String,
}

#[derive(Args, Clone, Debug)]
pub struct Delete {
    #[command(flatten)]
    pub identifier: Identifier,

    /// Confirm deletion
    #[arg(long)]
    pub confirm: bool,
}
