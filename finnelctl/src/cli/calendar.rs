use clap::{Args, Subcommand};

#[derive(Debug, Clone, Subcommand)]
pub enum Command {
    /// Show the calendar
    Show(Show),
    /// Show report for today
    Today(Today),
}

#[derive(Default, Args, Clone, Debug)]
pub struct Show {}

#[derive(Default, Args, Clone, Debug)]
pub struct Today {}
