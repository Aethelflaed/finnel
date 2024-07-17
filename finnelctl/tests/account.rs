#[macro_use]
mod common;
use common::prelude::*;

#[test]
fn empty() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, account).failure().stderr(str::contains("Usage:"));

    Ok(())
}

#[test]
fn list() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, account create Cash).success();
    cmd!(env, account create Bank).success();

    cmd!(env, account list)
        .success()
        .stdout(str::contains("€ 0.00"))
        .stdout(str::contains("1  | Cash"))
        .stdout(str::contains("2  | Bank"));

    Ok(())
}

#[test]
fn create() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, account create)
        .failure()
        .stderr(str::contains("  <NAME>"));

    cmd!(env, account create Cash)
        .success()
        .stdout(str::is_empty());

    Ok(())
}

#[test]
fn show() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, account create Cash).success();

    cmd!(env, account show)
        .failure()
        .stderr(str::contains("Account not provided"));

    cmd!(env, account show -A Bank)
        .failure()
        .stderr(str::contains("Account not found"));

    cmd!(env, account show -A Cash)
        .success()
        .stdout(str::contains("1 | Cash"))
        .stdout(str::contains("Balance: € 0.00"));

    cmd!(env, account default -A Cash).success();

    cmd!(env, account show)
        .success()
        .stdout(str::contains("1 | Cash"))
        .stdout(str::contains("Balance: € 0.00"));

    Ok(())
}

#[test]
fn delete() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, account create Cash).success();

    cmd!(env, account delete)
        .failure()
        .stderr(str::contains("Account not provided"));

    cmd!(env, account delete -A Cash)
        .failure()
        .stdout(str::is_empty())
        .stderr(str::contains("requires confirmation"));

    cmd!(env, account delete -A Cash --confirm)
        .failure()
        .stdout("Do you really want to do that?\n")
        .stderr(str::contains("requires confirmation"));

    raw_cmd!(env, account delete -A Cash --confirm)
        .write_stdin("no")
        .assert()
        .failure()
        .stdout("Do you really want to do that?\n")
        .stderr(str::contains("requires confirmation"));

    raw_cmd!(env, account delete -A Cash --confirm)
        .write_stdin("yes")
        .assert()
        .success()
        .stdout("Do you really want to do that?\n");

    cmd!(env, account show -A Cash)
        .failure()
        .stderr(str::contains("Account not found"));

    Ok(())
}

#[test]
fn default() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, account create Cash).success();

    cmd!(env, account default)
        .success()
        .stdout(str::contains("<not set>"));

    cmd!(env, account default --reset)
        .success()
        .stdout(str::is_empty());

    cmd!(env, account default -A Cash)
        .success()
        .stdout(str::is_empty());

    cmd!(env, account default)
        .success()
        .stdout(str::contains("Cash"));

    cmd!(env, account default --reset)
        .success()
        .stdout(str::is_empty());

    cmd!(env, account default)
        .success()
        .stdout(str::contains("<not set>"));

    Ok(())
}
