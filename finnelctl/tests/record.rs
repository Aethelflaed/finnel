#[macro_use]
mod common;
use common::prelude::*;

mod record {
    mod add;
}

pub fn setup(env: &crate::Env) -> Result<()> {
    cmd!(env, account create Cash).success();
    cmd!(env, account default -A Cash).success();

    Ok(())
}

#[test]
fn empty() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, account).failure().stderr(str::contains("Usage:"));

    Ok(())
}

#[test]
fn list() -> Result<()> {
    let env = Env::new()?;
    crate::setup(&env)?;

    cmd!(env, record add 10 bread).success();
    cmd!(env, record list --no_category)
        .success()
        .stdout(str::contains("bread"));

    Ok(())
}
