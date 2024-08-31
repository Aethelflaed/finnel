#[macro_use]
mod common;
use common::prelude::*;

mod record {
    mod create;
    mod list;
    mod split;
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
