use std::str::FromStr;

use super::{Boursobank, Importer, Logseq, Options};
use crate::config::Config;

use anyhow::Result;
use chrono::NaiveDate;

pub trait Profile {
    fn run(&mut self, importer: &mut Importer) -> Result<()>;
}

#[derive(Clone, Debug, PartialEq)]
pub enum Information {
    Logseq,
    Boursobank,
    None,
    #[cfg(test)]
    Test,
}

impl Default for Information {
    fn default() -> Self {
        #[cfg(test)]
        return Information::Test;
        #[cfg(not(test))]
        return Information::None;
    }
}

impl FromStr for Information {
    type Err = anyhow::Error;

    fn from_str(name: &str) -> Result<Self> {
        match name.to_lowercase().as_str() {
            "logseq" => Ok(Information::Logseq),
            "boursobank" => Ok(Information::Boursobank),
            #[cfg(test)]
            "test" => Ok(Information::Test),
            _ => anyhow::bail!("Unknown profile '{}'", name),
        }
    }
}

impl Information {
    pub fn new_profile(&self, options: &Options) -> Result<Box<dyn Profile>> {
        Ok(match self {
            Information::Boursobank => Box::new(Boursobank::new(options)?),
            Information::Logseq => Box::new(Logseq::new(options)?),
            Information::None => anyhow::bail!("Profile not set"),
            #[cfg(test)]
            Information::Test => anyhow::bail!("test profile"),
        })
    }

    pub fn name(&self) -> Result<&str> {
        Ok(match self {
            Information::Boursobank => "boursobank",
            Information::Logseq => "logseq",
            Information::None => anyhow::bail!("Profile not set"),
            #[cfg(test)]
            Information::Test => "test",
        })
    }

    pub fn last_imported(&self, config: &Config) -> Result<Option<NaiveDate>> {
        Ok(self
            .get(config, "last_imported")?
            .map(|value| value.parse())
            .transpose()?)
    }

    pub fn set_last_imported(&self, config: &Config, date: Option<NaiveDate>) -> Result<()> {
        if let Some(date) = date {
            if let Some(previous_date) = self.last_imported(config).ok().flatten() {
                if previous_date > date {
                    anyhow::bail!(
                        "Cannot set last_imported. Given {} is before current {}",
                        date,
                        previous_date
                    );
                }
            }

            self.set(config, "last_imported", date.to_string().as_str())
        } else {
            self.reset(config, "last_imported")
        }
    }

    pub fn default_account(&self, config: &Config) -> Result<Option<String>> {
        self.get(config, "default_account")
    }

    pub fn set_default_account(&self, config: &Config, account: Option<&str>) -> Result<()> {
        if let Some(account) = account {
            self.set(config, "default_account", account)
        } else {
            self.reset(config, "default_account")
        }
    }

    fn get(&self, config: &Config, key: &str) -> Result<Option<String>> {
        config.get(format!("{}/{}", self.name()?, key).as_str())
    }

    fn set(&self, config: &Config, key: &str, value: &str) -> Result<()> {
        config.set(format!("{}/{}", self.name()?, key).as_str(), value)
    }

    fn reset(&self, config: &Config, key: &str) -> Result<()> {
        config.reset(format!("{}/{}", self.name()?, key).as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::prelude::{assert_eq, Result, *};

    #[test]
    fn parse() -> Result<()> {
        assert_eq!(Information::Boursobank, "Boursobank".parse()?);
        assert!("".parse::<Information>().is_err());

        Ok(())
    }

    #[test]
    fn last_imported() -> Result<()> {
        let profile = Information::Boursobank;

        with_config(|config| {
            assert!(profile.last_imported(config)?.is_none());

            let date = chrono::Utc::now().date_naive();
            let past_date = date - chrono::naive::Days::new(1);

            profile.set_last_imported(config, Some(past_date))?;
            profile.set_last_imported(config, Some(date))?;

            assert!(profile.set_last_imported(config, Some(past_date)).is_err());

            assert_eq!(Some(date), profile.last_imported(config)?);

            profile.set_last_imported(config, None)?;
            assert!(profile.last_imported(config)?.is_none());

            Ok(())
        })
    }

    #[test]
    fn default_account() -> Result<()> {
        with_config(|config| {
            let profile = Information::default();
            assert!(profile.default_account(config)?.is_none());

            profile.set_default_account(config, Some("foo"))?;
            assert_eq!(Some("foo".to_owned()), profile.default_account(config)?);

            profile.set_default_account(config, None)?;
            assert!(profile.default_account(config)?.is_none());
            Ok(())
        })
    }
}
