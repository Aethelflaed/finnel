use anyhow::Result;
use finnel::prelude::*;

pub mod prelude {
    pub use crate::test::{self, with::*};
    pub use anyhow::Result;
    pub use assert_fs::fixture::{FileWriteStr, PathChild};
    pub use finnel::prelude::*;
    pub use pretty_assertions::assert_eq;
}

pub fn conn() -> Result<Conn> {
    let mut conn = finnel::Database::memory()?;
    conn.setup()?;
    Ok(conn.into())
}

pub fn conn_file(path: &std::path::Path) -> Result<Conn> {
    let mut conn = finnel::Database::open(path)?;
    conn.setup()?;
    Ok(conn.into())
}

pub fn account(conn: &mut Conn, name: &str) -> Result<Account> {
    Ok(finnel::account::NewAccount::new(name).save(conn)?)
}

pub fn category(conn: &mut Conn, name: &str) -> Result<Category> {
    Ok(finnel::category::NewCategory::new(name).save(conn)?)
}

pub fn merchant(conn: &mut Conn, name: &str) -> Result<Merchant> {
    Ok(finnel::merchant::NewMerchant::new(name).save(conn)?)
}

pub fn record(conn: &mut Conn, account: &Account) -> Result<Record> {
    Ok(finnel::record::NewRecord::new(account).save(conn)?)
}

pub mod with {
    use super::Result;
    use crate::config::Config;

    pub fn with_temp_dir<F, R>(function: F) -> R
    where
        F: FnOnce(&assert_fs::TempDir) -> R,
    {
        let temp = assert_fs::TempDir::new()
            .unwrap()
            .into_persistent_if(std::env::var_os("TEST_PERSIST_FILES").is_some());
        let result = function(&temp);

        // The descrutor would silence any issue, so we call close() explicitly
        temp.close().unwrap();

        result
    }

    pub fn with_config_dir<F, R>(function: F) -> R
    where
        F: FnOnce(&assert_fs::TempDir) -> R,
    {
        with_temp_dir(|temp| {
            temp_env::with_var("FINNEL_CONFIG", Some(temp.path().as_os_str()), || {
                function(temp)
            })
        })
    }

    pub fn with_data_dir<F, R>(function: F) -> R
    where
        F: FnOnce(&assert_fs::TempDir) -> R,
    {
        with_temp_dir(|temp| {
            temp_env::with_var("FINNEL_DATA", Some(temp.path().as_os_str()), || {
                function(temp)
            })
        })
    }

    pub fn with_dirs<F, R>(function: F) -> R
    where
        F: FnOnce(&assert_fs::TempDir, &assert_fs::TempDir) -> R,
    {
        with_config_dir(|config| with_data_dir(|data| function(config, data)))
    }

    pub fn with_config<F, R>(function: F) -> Result<R>
    where
        F: FnOnce(&Config) -> Result<R>,
    {
        with_config_args(&[], function)
    }

    pub fn with_config_args<F, R>(additional_args: &[&str], function: F) -> Result<R>
    where
        F: FnOnce(&Config) -> Result<R>,
    {
        with_dirs(|confd, datad| {
            let mut args = vec![
                "arg0",
                "--config",
                confd.path().to_str().unwrap(),
                "--data",
                datad.path().to_str().unwrap(),
            ];

            args.extend(additional_args);

            let config = Config::try_parse_from(args.as_slice())?;

            function(&config)
        })
    }

    pub fn with_fixtures<F, R>(patterns: &[&str], function: F) -> Result<R>
    where
        F: FnOnce(&assert_fs::TempDir) -> Result<R>,
    {
        use assert_fs::fixture::PathCopy;
        use std::path::PathBuf;

        let fixtures_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures");

        with_temp_dir(|dir| {
            dir.copy_from(fixtures_path, patterns)?;

            function(dir)
        })
    }
}

mod tests {
    use super::*;

    #[test]
    fn with_config() -> Result<()> {
        with::with_config(|config| {
            assert!(config.dir.exists());
            assert!(config.data_dir.exists());

            Ok(())
        })
    }

    #[test]
    fn with_fixtures() -> Result<()> {
        use assert_fs::fixture::PathChild;

        let file = "boursobank/curated.csv";
        with::with_fixtures(&[file], |dir| {
            assert!(dir.child(file).path().exists());

            Ok(())
        })
    }
}
