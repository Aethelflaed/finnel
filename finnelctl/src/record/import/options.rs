use std::path::PathBuf;

use super::ProfileInformation;
use crate::cli::record::Import as ImportOptions;
use crate::config::Config;

use anyhow::Result;
use chrono::{offset::Utc, DateTime};

#[derive(Default, Clone, Debug)]
pub struct Options {
    pub file: PathBuf,
    pub profile: ProfileInformation,
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}

impl Options {
    pub fn try_from(cli: &ImportOptions, config: &Config) -> Result<Self> {
        let profile = cli.profile.parse::<ProfileInformation>()?;

        let from = cli
            .from()?
            .or_else(|| profile.last_imported(config).ok().flatten());

        Ok(Self {
            file: cli.file.clone(),
            profile,
            from,
            to: cli.to()?.or_else(|| Some(Utc::now())),
        })
    }

    pub fn set_last_imported_unchecked(&self, config: &Config, date: DateTime<Utc>) {
        let _ = self.profile.set_last_imported(config, Some(date));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{Cli, record::Command, Commands};
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

                assert_eq!(ProfileInformation::Boursobank, options.profile);
                assert_eq!(
                    Some(parse_date_fmt("2024-07-01", "%Y-%m-%d")?),
                    options.from
                );
                assert_eq!(Some(parse_date_fmt("2024-07-31", "%Y-%m-%d")?), options.to);

                let date = parse_date_fmt("2024-08-01", "%Y-%m-%d")?;
                options.set_last_imported_unchecked(config, date);

                let cli = Cli::try_parse_from(&[
                    "arg0",
                    "record",
                    "import",
                    "-P",
                    "BoursoBank",
                    "FILE",
                ])?;
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
