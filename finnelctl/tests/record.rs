mod common;
use common::prelude::*;

mod record {
    mod add;
}

pub fn setup(env: &crate::Env) -> Result<()> {
    env.command()?
        .arg("account")
        .arg("create")
        .arg("Cash")
        .assert()
        .success()
        .stdout(str::is_empty());

    env.command()?
        .arg("account")
        .arg("default")
        .arg("-A")
        .arg("Cash")
        .assert()
        .success()
        .stdout(str::is_empty());

    Ok(())
}
