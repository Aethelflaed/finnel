use clap::Subcommand;

#[derive(Debug, Clone, Subcommand)]
pub enum AccountCommands {
    /// List registered accounts
    List {},
    /// Create a new account
    Create {
        /// Name of the new account
        account_name: String,
    },
    /// Show details about an account
    Show {},
    /// Delete an account
    Delete {
        /// Confirm deletion
        #[arg(long, hide = true)]
        confirm: bool,
    },
    /// Check or set the default account
    Default {
        /// Reset the default account
        #[arg(short, long)]
        reset: bool,
    },
}

