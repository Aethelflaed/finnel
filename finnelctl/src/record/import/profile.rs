use std::str::FromStr;

use crate::config::Config;

use anyhow::Result;
use chrono::{offset::Utc, DateTime};

#[derive(Default, Clone, Debug, PartialEq)]
pub enum ProfileInformation {
    Boursobank,
    #[default]
    None,
}

impl FromStr for ProfileInformation {
    type Err = anyhow::Error;

    fn from_str(name: &str) -> Result<Self> {
        match name.to_lowercase().as_str() {
            "boursobank" => Ok(ProfileInformation::Boursobank),
            _ => anyhow::bail!("Unknown profile '{}'", name),
        }
    }
}

impl ProfileInformation {
    pub fn name(&self) -> Result<&str> {
        Ok(match self {
            ProfileInformation::Boursobank => "boursobank",
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
        assert_eq!(ProfileInformation::Boursobank, "Boursobank".parse()?);
        assert!("".parse::<ProfileInformation>().is_err());

        Ok(())
    }

    #[test]
    fn last_imported() -> Result<()> {
        let profile = ProfileInformation::Boursobank;

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
