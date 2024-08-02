use std::str::FromStr;

use super::{Boursobank, Importer, Logseq, Options};
use crate::config::Config;

use anyhow::Result;
use chrono::{offset::Utc, DateTime};

pub trait Profile {
    fn run(&mut self, importer: &mut Importer) -> Result<()>;
}

#[derive(Default, Clone, Debug, PartialEq)]
pub enum Information {
    Logseq,
    Boursobank,
    #[default]
    None,
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
            _ => anyhow::bail!("Profile not set"),
        })
    }

    pub fn name(&self) -> Result<&str> {
        Ok(match self {
            Information::Boursobank => "boursobank",
            _ => anyhow::bail!("Profile not set"),
        })
    }

    pub fn last_imported(&self, config: &Config) -> Result<Option<DateTime<Utc>>> {
        Ok(config
            .get(format!("{}/last_imported", self.name()?).as_str())?
            .map(|value| value.parse())
            .transpose()?)
    }

    pub fn set_last_imported(&self, config: &Config, date: Option<DateTime<Utc>>) -> Result<()> {
        if let Some(date) = date {
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

            let date = Utc::now();

            profile.set_last_imported(config, Some(date))?;

            assert_eq!(Some(date), profile.last_imported(config)?);

            profile.set_last_imported(config, None)?;
            assert!(profile.last_imported(config)?.is_none());

            Ok(())
        })
    }
}
