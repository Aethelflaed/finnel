use std::path::PathBuf;

use super::{Information, Profile};
use crate::cli::record::Import as ImportOptions;
use crate::config::Config;

use anyhow::Result;
use chrono::{offset::Utc, DateTime};

#[derive(Clone, Debug)]
pub struct Options<'a> {
    pub config: &'a Config,
    pub file: PathBuf,
    pub profile_info: Information,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub print: bool,
    pub pretend: bool,
}

impl<'a> Options<'a> {
    pub fn new(config: &'a Config) -> Self {
        Options {
            config,
            file: Default::default(),
            profile_info: Default::default(),
            from: Default::default(),
            to: Default::default(),
            print: false,
            pretend: false,
        }
    }

    pub fn try_from(cli: &ImportOptions, config: &'a Config) -> Result<Self> {
        let profile_info = cli.profile.parse::<Information>()?;

        let from = cli
            .from()?
            .or_else(|| {
                let from = profile_info.last_imported(config).ok().flatten();
                if let Some(date) = from {
                    log::info!("Starting import from last imported date: {}", date);
                }
                from
            });

        Ok(Self {
            config,
            file: cli.file.clone(),
            profile_info,
            from,
            to: cli.to()?.or_else(|| Some(Utc::now())),
            print: cli.print,
            pretend: cli.pretend,
        })
    }

    pub fn new_profile(&self) -> Result<Box<dyn Profile>> {
        self.profile_info.new_profile(self)
    }

    pub fn last_imported(&self) -> Result<Option<DateTime<Utc>>> {
        self.profile_info.last_imported(self.config)
    }

    pub fn set_last_imported(&self, date: DateTime<Utc>) -> Result<()> {
        if let Some(previous_date) = self.last_imported().ok().flatten() {
            if previous_date > date {
                return Ok(());
            }
        }

        self.profile_info.set_last_imported(self.config, Some(date))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{record::Command, Cli, Commands};
    use crate::record::import::parse_date_fmt;
    use crate::test::prelude::{assert_eq, *};

    use clap::Parser;

    #[test]
    fn construction() -> Result<()> {
        with_config_args(
            &[
                "record",
                "import",
                "-P",
                "BoursoBank",
                "FILE",
                "--from",
                "2024-07-01",
                "--to",
                "2024-07-31",
            ],
            |config| {
                let Some(Commands::Record(Command::Import(import))) = config.command() else {
                    panic!("Unexpected CLI parse")
                };

                let options = Options::try_from(import, config)?;

                assert_eq!(Information::Boursobank, options.profile_info);
                assert_eq!(
                    Some(parse_date_fmt("2024-07-01", "%Y-%m-%d")?),
                    options.from
                );
                assert_eq!(Some(parse_date_fmt("2024-07-31", "%Y-%m-%d")?), options.to);

                let date = parse_date_fmt("2024-08-01", "%Y-%m-%d")?;
                options.set_last_imported(date)?;

                // check that using a previous date afterwards does not change the last_imported
                // and error is silently ignored
                options.set_last_imported(date - core::time::Duration::from_secs(86400))?;

                let cli =
                    Cli::try_parse_from(&["arg0", "record", "import", "-P", "BoursoBank", "FILE"])?;
                let Some(Commands::Record(Command::Import(import))) = cli.command.as_ref() else {
                    panic!("Unexpected CLI parse")
                };

                let options = Options::try_from(import, config)?;

                assert_eq!(Some(date), options.from);
                let to = options.to.unwrap();
                assert!(to - Utc::now() < chrono::TimeDelta::new(1, 0).unwrap());

                Ok(())
            },
        )
    }
}
