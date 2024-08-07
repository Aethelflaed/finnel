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

#[test]
fn pretend() -> Result<()> {
    let env = Env::new()?;
    crate::setup(&env)?;

    let csv = "boursobank/curated.csv";
    env.copy_fixtures(&[csv])?;

    raw_cmd!(env, record import -P Boursobank --pretend)
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
    crate::setup(&env)?;

    let csv = "boursobank/curated.csv";
    env.copy_fixtures(&[csv])?;

    raw_cmd!(env, record import -P Boursobank --print)
        .arg(env.data_dir.child(csv).as_os_str())
        .assert()
        .success()
        .stdout(str::contains("LE CHARIOT"));

    cmd!(env, record show 1).success();

    Ok(())
}
