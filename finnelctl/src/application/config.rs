use std::path::PathBuf;

use anyhow::Result;
use toml::{Table, Value};

use crate::application::cli::Cli;

pub struct Config {
    dir: PathBuf,
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

        Ok(Config {
            dir,
            data_dir,
            cli,
            table,
        })
    }
}

fn config_home() -> PathBuf {
    match std::env::var("FINNEL_CONFIG") {
        Ok(val) if !val.is_empty() => PathBuf::from(val),
        _ => xdg::BaseDirectories::with_prefix("finnel")
            .unwrap()
            .get_config_home(),
    }
}

fn data_home() -> PathBuf {
    match std::env::var("FINNEL_DATA") {
        Ok(val) if !val.is_empty() => PathBuf::from(val),
        _ => xdg::BaseDirectories::with_prefix("finnel")
            .unwrap()
            .get_data_home(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::{assert_eq, with_dirs};
    use assert_fs::fixture::{FileWriteStr, PathChild};

    #[test]
    fn parse() -> Result<()> {
        with_dirs(|confd, datad| {
            let mut config = Config::try_parse()?;
            assert_eq!(config.dir, confd.path());
            assert_eq!(config.data_dir, datad.path());

            confd.child("config.toml").write_str(&format!(
                "data_dir = '{}'",
                datad.child("foo").path().display()
            ))?;

            config = Config::try_parse()?;
            assert_eq!(config.data_dir, datad.child("foo").path());

            config = Config::try_parse_from(&[
                "arg0",
                "--config",
                datad.child("bar").path().to_str().unwrap(),
            ])?;
            assert_eq!(config.dir, datad.child("bar").path());

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