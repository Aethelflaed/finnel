use std::path::PathBuf;

use super::{Information, Profile};
use crate::cli::record::Import as ImportOptions;
use crate::config::Config;

use anyhow::Result;
use chrono::{NaiveDate, Utc, Days};

#[derive(Clone, Debug)]
pub struct Options<'a> {
    pub config: &'a Config,
    pub file: PathBuf,
    pub profile_info: Information,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
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

        let from = cli.from.or_else(|| {
            let from = profile_info.last_imported(config).ok().flatten();
            if let Some(date) = from {
                log::info!("Starting import from last imported date: {}", date);
                Some(date + Days::new(1))
            } else {
                None
            }
        });

        Ok(Self {
            config,
            file: cli.file.clone(),
            profile_info,
            from,
            to: cli.to.or_else(|| Some(Utc::now().date_naive())),
            print: cli.print,
            pretend: cli.pretend,
        })
    }

    pub fn new_profile(&self) -> Result<Box<dyn Profile>> {
        self.profile_info.new_profile(self)
    }

    pub fn last_imported(&self) -> Result<Option<NaiveDate>> {
        self.profile_info.last_imported(self.config)
    }

    pub fn set_last_imported(&self, date: Option<NaiveDate>) -> Result<()> {
        if self.pretend {
            return Ok(());
        }

        if let Some(date) = date {
            if let Some(previous_date) = self.last_imported().ok().flatten() {
                if previous_date > date {
                    return Ok(());
                }
            }
        }

        self.profile_info.set_last_imported(self.config, date)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::{record::Command, Commands};
    use crate::test::prelude::{assert_eq, *};

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

                // Check that CLI information is correctly read
                assert_eq!(Information::Boursobank, options.profile_info);
                assert_eq!(NaiveDate::from_ymd_opt(2024, 7, 1), options.from);
                assert_eq!(NaiveDate::from_ymd_opt(2024, 7, 31), options.to);

                Ok(())
            },
        )
    }

    #[test]
    fn use_last_imported_if_from_is_absent() -> Result<()> {
        with_config_args(&["record", "import", "-P", "Test", "FILE"], |config| {
            let date = NaiveDate::from_ymd_opt(2024, 8, 1);

            {
                // Set the last imported using a different object
                let options = Options::new(config);
                options.set_last_imported(date)?;
            }

            let Some(Commands::Record(Command::Import(import))) = config.command() else {
                panic!("Unexpected CLI parse")
            };
            let options = Options::try_from(import, config)?;

            assert_eq!(date.unwrap() + Days::new(1), options.from.unwrap());

            // Also check that to is set to today
            let to = options.to.unwrap();
            assert!(to - Utc::now().date_naive() == chrono::Duration::days(0));

            Ok(())
        })
    }

    #[test]
    fn last_imported() -> Result<()> {
        with_config(|config| {
            let mut options = Options::new(config);
            // Make sure it's not the none profile, otherwise the rest fails. Also suppress a warning
            // present only when compiling tests
            assert_ne!(options.profile_info, Information::None);

            assert_eq!(None, options.last_imported()?);

            let date = NaiveDate::from_ymd_opt(2024, 8, 1);
            let next_day = date.map(|d| d + chrono::Days::new(1));

            options.set_last_imported(date)?;
            assert_eq!(date, options.last_imported()?);

            // Nothing is set if pretend
            options.pretend = true;
            options.set_last_imported(next_day)?;
            assert_eq!(date, options.last_imported()?);

            options.pretend = false;
            options.set_last_imported(next_day)?;
            assert_eq!(next_day, options.last_imported()?);

            // Setting a previous date is silently ignored
            options.set_last_imported(date)?;
            assert_eq!(next_day, options.last_imported()?);

            Ok(())
        })
    }
}
