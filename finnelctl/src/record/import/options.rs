use std::path::PathBuf;

use super::{Information, Profile};
use crate::cli::record::Import as ImportOptions;
use crate::config::Config;

use anyhow::Result;
use chrono::{Days, NaiveDate, Utc};

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
        let today = Utc::now().date_naive();

        let from = if let Some(from) = cli.from {
            if from > today {
                log::warn!(
                    "--from cannot be in the future, changing  to today {}",
                    today
                );
                Some(today)
            } else {
                Some(from)
            }
        } else {
            let from = profile_info.last_imported(config)?;
            if let Some(date) = from {
                if date < today {
                    // Add one day or we might re-import the same day
                    let date = date + Days::new(1);
                    log::info!("Starting import from last imported date + 1 day: {}", date);
                    Some(date)
                } else {
                    log::info!("Starting import from today {}", today);
                    Some(today)
                }
            } else {
                None
            }
        };

        Ok(Self {
            config,
            file: cli.file.clone(),
            profile_info,
            from,
            to: cli.to.or_else(|| Some(today)),
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
    fn use_today_if_last_imported_is_in_the_future() -> Result<()> {
        with_config_args(&["record", "import", "-P", "Test", "FILE"], |config| {
            let today = Utc::now().date_naive();

            {
                let options = Options::new(config);
                options.set_last_imported(Some(today + Days::new(3)))?;
            }

            let Some(Commands::Record(Command::Import(import))) = config.command() else {
                panic!("Unexpected CLI parse")
            };
            let options = Options::try_from(import, config)?;

            assert_eq!(today, options.from.unwrap());
            Ok(())
        })
    }

    #[test]
    fn use_today_if_from_is_in_the_future() -> Result<()> {
        let date = Utc::now().date_naive() + Days::new(3);
        with_config_args(
            &[
                "record",
                "import",
                "-P",
                "Test",
                "FILE",
                "--from",
                date.to_string().as_str(),
            ],
            |config| {
                let Some(Commands::Record(Command::Import(import))) = config.command() else {
                    panic!("Unexpected CLI parse")
                };

                let options = Options::try_from(import, config)?;
                assert_eq!(Utc::now().date_naive(), options.from.unwrap());

                Ok(())
            },
        )
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
