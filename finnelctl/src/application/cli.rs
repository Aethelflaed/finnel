use std::path::PathBuf;

use clap::Parser;

/// Finnel control
#[derive(Default, Debug, Parser)]
#[command(version)]
pub struct Cli {
    #[clap(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,

    /// Sets a custom config directory
    ///
    /// The default value is $FINNEL_CONFIG if it is set, or
    /// $XDG_CONFIG_HOME/finnel otherwise
    #[arg(short = 'C', long, value_name = "DIR")]
    pub config: Option<PathBuf>,

    /// Sets a custom data directory
    ///
    /// The default value is $FINNEL_DATA if it is set, or
    /// $XDG_DATA_HOME/finnel otherwise
    #[arg(short = 'D', long, value_name = "DIR")]
    pub data: Option<PathBuf>,
}
