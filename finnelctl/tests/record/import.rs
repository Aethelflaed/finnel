use crate::common::prelude::*;

#[test]
fn empty() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, record import)
        .failure()
        .stderr(str::contains("Usage:"))
        .stderr(str::contains("\n  --profile <PROFILE>"))
        .stderr(str::contains("\n  <FILE>"));

    Ok(())
}

#[test]
fn account_required() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, record import foo --profile unknown)
        .failure()
        .stderr(str::contains("Account not provided"));

    Ok(())
}

#[test]
fn unknown_profile() -> Result<()> {
    let env = Env::new()?;
    crate::setup(&env)?;

    cmd!(env, record import foo --profile unknown)
        .failure()
        .stderr(str::contains("Unknown profile 'unknown'"));

    Ok(())
}
