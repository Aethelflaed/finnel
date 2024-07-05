use anyhow::Result;
use assert_cmd::Command;
use assert_fs::TempDir;

pub mod prelude {
    pub use super::Env;
    pub use anyhow::Result;
    #[allow(unused_imports)]
    pub use predicates::prelude::*;
    pub use predicates::str;
}

pub struct Env {
    pub conf_dir: TempDir,
    pub data_dir: TempDir,
}

#[allow(unused_macros)]
macro_rules! cmd {
    ($env:ident, $($tail:tt)*) => {
        cmd!(@args $env.command()?, $($tail)* )
    };
    (@args $cmd:expr, --$arg:tt) => {
        $cmd.arg(concat!("--", stringify!($arg))).assert()
    };
    (@args $cmd:expr, $arg:tt) => {
        $cmd.arg(stringify!($arg)).assert()
    };
    (@args $cmd:expr, --$arg:tt $($tail:tt)*) => {
        cmd!(@args $cmd.arg(cmd!(@arg --$arg)), $($tail)*)
    };
    (@args $cmd:expr, $arg:tt $($tail:tt)*) => {
        cmd!(@args $cmd.arg(cmd!(@arg $arg)), $($tail)*)
    };
    (@arg --$arg:tt) => { concat!("--", stringify!($arg)) };
    (@arg $arg:tt) => { stringify!($arg) };
}

impl Env {
    pub fn new() -> Result<Self> {
        Ok(Self {
            conf_dir: TempDir::new()?,
            data_dir: TempDir::new()?,
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
}
