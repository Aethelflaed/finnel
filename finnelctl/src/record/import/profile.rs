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
        Ok(config
            .get(format!("{}/last_imported", self.name()?).as_str())?
            .map(|value| value.parse())
            .transpose()?)
    }

    pub fn set_last_imported(&self, config: &Config, date: Option<NaiveDate>) -> Result<()> {
        if let Some(date) = date {
            if let Some(previous_date) = self.last_imported(config).ok().flatten() {
                if previous_date > date {
                    anyhow::bail!("Cannot set last_imported to ");
                }
            }

            config.set(
                format!("{}/last_imported", self.name()?).as_str(),
                date.to_string().as_str(),
            )
        } else {
            config.reset(format!("{}/last_imported", self.name()?).as_str())
        }
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
}
