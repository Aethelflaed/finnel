use std::fs::create_dir;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use toml::{Table, Value};

use finnel::{Account, Database, Error};

use crate::cli::{Cli, Commands};

pub struct Config {
    _dir: PathBuf,
    data_dir: PathBuf,
    cli: Cli,
    table: Table,
}

impl Config {
    pub fn try_parse() -> Result<Self> {
        Self::try_parse_from(std::env::args_os())
    }

    fn try_parse_from<I, T>(iter: I) -> Result<Self>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        use clap::Parser;

        let cli = Cli::parse_from(iter);

        let dir = cli.config.clone().unwrap_or_else(config_home);
        let table = match std::fs::read_to_string(dir.join("config.toml")) {
            Ok(content) => content.parse::<Table>()?,
            Err(_) => Table::new(),
        };

        let data_dir = cli.data.clone().unwrap_or_else(|| {
            table
                .get("data_dir")
                .and_then(Value::as_str)
                .map(PathBuf::from)
                .unwrap_or_else(data_home)
        });

        if !data_dir.is_dir() {
            return Err(anyhow!(
                "Data directory is not a dir: {}",
                data_dir.display()
            ));
        }

        Ok(Config {
            _dir: dir,
            data_dir,
            cli,
            table,
        })
    }

    pub fn account_name(&self) -> Option<&str> {
        self.cli.account.as_deref()
    }

    pub fn account_or_default(&self, db: &Database) -> Result<Account> {
        if let Some(name) = self.account_name() {
            match Account::find_by_name(db, name) {
                Ok(account) => Ok(account),
                Err(Error::NotFound) => {
                    Err(anyhow!("Account not found: {}", name))
                }
                Err(e) => Err(e.into()),
            }
        } else {
            match crate::account::default(db) {
                Ok(None) => Err(anyhow!("Account not provided")),
                Ok(Some(account)) => Ok(account),
                Err(e) => Err(e),
            }
        }
    }

    pub fn command(&self) -> &Option<Commands> {
        &self.cli.command
    }

    pub fn database_path(&self) -> PathBuf {
        let db_filename = if let Some(db_table) =
            self.table.get("db").and_then(Value::as_table)
        {
            db_table
                .get("filename")
                .and_then(Value::as_str)
                .unwrap_or("db.finnel")
        } else {
            "db.finnel"
        };

        self.data_dir.join(db_filename)
    }

    pub fn database(&self) -> Result<Database> {
        let db = Database::open(self.database_path())?;
        db.setup()?;
        Ok(db)
    }
}

fn config_home() -> PathBuf {
    match std::env::var("FINNEL_CONFIG") {
        Ok(val) if !val.is_empty() => PathBuf::from(val),
        _ => {
            let path = xdg::BaseDirectories::with_prefix("finnel")
                .unwrap()
                .get_config_home();
            if !path.exists() {
                create_dir(&path).unwrap();
            }
            path
        }
    }
}

fn data_home() -> PathBuf {
    match std::env::var("FINNEL_DATA") {
        Ok(val) if !val.is_empty() => PathBuf::from(val),
        _ => {
            let path = xdg::BaseDirectories::with_prefix("finnel")
                .unwrap()
                .get_data_home();
            if !path.exists() {
                create_dir(&path).unwrap();
            }
            path
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::prelude::{assert_eq, *};

    #[test]
    fn parse() -> Result<()> {
        with_dirs(|confd, datad| {
            let mut config = Config::try_parse()?;
            assert_eq!(config._dir, confd.path());
            assert_eq!(config.data_dir, datad.path());

            confd.child("config.toml").write_str(&format!(
                "data_dir = '{}'",
                datad.child("foo").path().display()
            ))?;

            assert!(Config::try_parse().is_err());
            let _ = create_dir(datad.child("foo").path());
            config = Config::try_parse()?;
            assert_eq!(config.data_dir, datad.child("foo").path());

            config = Config::try_parse_from(&[
                "arg0",
                "--config",
                datad.child("bar").path().to_str().unwrap(),
            ])?;
            assert_eq!(config._dir, datad.child("bar").path());

            let _ = create_dir(datad.child("bar").path());
            config = Config::try_parse_from(&[
                "arg0",
                "-D",
                datad.child("bar").path().to_str().unwrap(),
            ])?;
            assert_eq!(config.data_dir, datad.child("bar").path());

            Ok(())
        })
    }

    #[test]
    fn config_home_default() {
        temp_env::with_var("FINNEL_CONFIG", None::<&str>, || {
            let xdg_dirs = xdg::BaseDirectories::with_prefix("finnel").unwrap();
            assert_eq!(xdg_dirs.get_config_home(), config_home());
        });
    }

    #[test]
    fn config_home_with_var() {
        temp_env::with_var("FINNEL_CONFIG", Some("./"), || {
            assert_eq!(PathBuf::from("./"), config_home());
        });
    }

    #[test]
    fn data_home_default() {
        temp_env::with_var("FINNEL_DATA", None::<&str>, || {
            let xdg_dirs = xdg::BaseDirectories::with_prefix("finnel").unwrap();
            assert_eq!(xdg_dirs.get_data_home(), data_home());
        });
    }

    #[test]
    fn data_home_with_var() {
        temp_env::with_var("FINNEL_DATA", Some("./"), || {
            assert_eq!(PathBuf::from("./"), data_home());
        });
    }
}
