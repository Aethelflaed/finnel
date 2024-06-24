use anyhow::Result;
use assert_cmd::Command;
use assert_fs::TempDir;

pub mod prelude {
    pub use super::Env;
    pub use anyhow::Result;
    pub use predicates::prelude::*;
    pub use predicates::str;
}

pub struct Env {
    pub conf_dir: TempDir,
    pub data_dir: TempDir,
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
