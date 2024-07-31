#[macro_use]
mod common;
use common::prelude::*;

#[test]
fn empty() -> Result<()> {
    let env = Env::new()?;

    env.command()?
        .assert()
        .failure()
        .stderr(str::contains("No command provided"));

    Ok(())
}

#[test]
fn consolidate() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, consolidate).success().stdout(str::is_empty());

    Ok(())
}

#[test]
fn reset() -> Result<()> {
    let env = Env::new()?;
    // Do something to create the db
    cmd!(env, account create Cash).success();

    cmd!(env, reset).failure().stderr(str::contains("Usage:"));

    cmd!(env, reset - -confirm)
        .failure()
        .stdout(str::contains("you really want"))
        .stderr(str::contains("requires confirmation"));

    raw_cmd!(env, reset - -confirm)
        .write_stdin("yes")
        .assert()
        .success()
        .stdout(str::contains("you really want"));

    Ok(())
}
