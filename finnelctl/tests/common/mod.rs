#![allow(unused_macros, unused_imports)]

use anyhow::Result;
use assert_cmd::Command;
use assert_fs::TempDir;

pub mod prelude {
    pub use super::Env;
    pub use anyhow::Result;
    pub use assert_fs::prelude::*;
    pub use predicates::prelude::*;
    pub use predicates::str;
}

pub struct Env {
    pub conf_dir: TempDir,
    pub data_dir: TempDir,
}

macro_rules! cmd {
    ($env:ident, $($tail:tt)*) => {
        raw_cmd!($env, $($tail)*).assert()
    };
}

macro_rules! raw_cmd {
    ($env:ident, $($tail:tt)*) => {
        raw_cmd!(@args $env.command()?, $($tail)* )
    };

    (@args $cmd:expr, --$arg:tt) => {
        $cmd.arg(raw_cmd!(@arg --$arg))
    };
    (@args $cmd:expr, -$arg:tt) => {
        $cmd.arg(raw_cmd!(@arg -$arg))
    };
    (@args $cmd:expr, $arg:tt) => {
        $cmd.arg(raw_cmd!(@arg $arg))
    };

    (@args $cmd:expr, --$arg:tt $($tail:tt)*) => {
        raw_cmd!(@args $cmd.arg(raw_cmd!(@arg --$arg)), $($tail)*)
    };
    (@args $cmd:expr, -$arg:tt $($tail:tt)*) => {
        raw_cmd!(@args $cmd.arg(raw_cmd!(@arg -$arg)), $($tail)*)
    };
    (@args $cmd:expr, $arg:tt $($tail:tt)*) => {
        raw_cmd!(@args $cmd.arg(raw_cmd!(@arg $arg)), $($tail)*)
    };

    (@arg --$arg:tt) => {
        concat!("--", stringify!($arg)).to_string().replace("_", "-")
    };
    (@arg -$arg:tt) => {
        concat!("-", stringify!($arg))
    };
    (@arg $arg:tt) => {
        stringify!($arg)
    };
}

impl Env {
    pub fn new() -> Result<Self> {
        Ok(Self {
            conf_dir: TempDir::new()?
                .into_persistent_if(std::env::var_os("TEST_PERSIST_FILES").is_some()),
            data_dir: TempDir::new()?
                .into_persistent_if(std::env::var_os("TEST_PERSIST_FILES").is_some()),
        })
    }

    pub fn command(&self) -> Result<Command> {
        let mut cmd = Command::cargo_bin("finnelctl")?;
        cmd.arg("-C")
            .arg(self.conf_dir.path())
            .arg("-D")
            .arg(self.data_dir.path());
        Ok(cmd)
    }

    pub fn copy_fixtures(&self, patterns: &[&str]) -> Result<()> {
        use assert_fs::fixture::PathCopy;
        use std::path::PathBuf;

        let fixtures_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures");

        self.data_dir.copy_from(fixtures_path, patterns)?;

        Ok(())
    }
}
