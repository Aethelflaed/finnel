#[macro_use]
mod common;
use common::prelude::*;

pub fn setup(env: &crate::Env) -> Result<()> {
    cmd!(env, account create Cash).success();
    cmd!(env, account default -A Cash).success();

    Ok(())
}

#[test]
fn empty() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, import)
        .failure()
        .stderr(str::contains("Usage:"))
        .stderr(str::contains("\n  --profile <PROFILE>"))
        .stderr(str::contains("\n  <FILE>"));

    Ok(())
}

#[test]
fn account_required() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, import foo --profile logseq)
        .failure()
        .stderr(str::contains("Account not provided"));

    Ok(())
}

#[test]
fn unknown_profile() -> Result<()> {
    let env = Env::new()?;
    setup(&env)?;

    cmd!(env, import foo --profile unknown)
        .failure()
        .stderr(str::contains("Unknown profile 'unknown'"));

    Ok(())
}

#[test]
fn pretend() -> Result<()> {
    let env = Env::new()?;
    setup(&env)?;

    let csv = "boursobank/curated.csv";
    env.copy_fixtures(&[csv])?;

    raw_cmd!(env, import -P Boursobank --pretend)
        .arg(env.data_dir.child(csv).as_os_str())
        .assert()
        .failure()
        .stderr(str::contains("we are pretending"));

    cmd!(env, record show 1).failure();

    Ok(())
}

#[test]
fn print() -> Result<()> {
    let env = Env::new()?;
    setup(&env)?;

    let csv = "boursobank/curated.csv";
    env.copy_fixtures(&[csv])?;

    raw_cmd!(env, import -P Boursobank --print)
        .arg(env.data_dir.child(csv).as_os_str())
        .assert()
        .success()
        .stdout(str::contains("LE CHARIOT"));

    cmd!(env, record show 1).success();

    Ok(())
}
