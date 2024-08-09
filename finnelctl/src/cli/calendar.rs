use clap::{Args, Subcommand};

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// Show the calendar
    Show(Show),
}

#[derive(Default, Args, Clone, Debug)]
pub struct Show {}
