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
        .stderr(str::contains("\n  --profile <PROFILE>"));

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
fn default_account() -> Result<()> {
    let env = Env::new()?;
    setup(&env)?;

    let csv = "boursobank/curated.csv";
    env.copy_fixtures(&[csv])?;

    raw_cmd!(env, import -P BoursoBank get)
        .arg("default-account")
        .assert()
        .success()
        .stdout(str::is_empty());

    raw_cmd!(env, import -P Boursobank -vvv)
        .arg(env.data_dir.child(csv).as_os_str())
        .assert()
        .success();

    cmd!(env, account default --reset).success();

    raw_cmd!(env, import -P Boursobank -vvv)
        .arg(env.data_dir.child(csv).as_os_str())
        .assert()
        .failure()
        .stderr(str::contains("Account not provided"));

    raw_cmd!(env, import -P BoursoBank set)
        .arg("default-account")
        .arg("Cash")
        .assert()
        .success();

    raw_cmd!(env, import -P Boursobank)
        .arg(env.data_dir.child(csv).as_os_str())
        .assert()
        .success();

    raw_cmd!(env, import -P BoursoBank get)
        .arg("default-account")
        .assert()
        .success()
        .stdout("Cash\n");

    raw_cmd!(env, import -P BoursoBank reset)
        .arg("default-account")
        .assert()
        .success();

    raw_cmd!(env, import -P Boursobank)
        .arg(env.data_dir.child(csv).as_os_str())
        .assert()
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
