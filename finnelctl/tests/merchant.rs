#[macro_use]
mod common;
use common::prelude::*;

#[test]
fn empty() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, merchant)
        .failure()
        .stderr(str::contains("Usage:"));

    Ok(())
}

#[test]
fn list() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, merchant create Chariot)
        .success()
        .stdout(str::is_empty());

    cmd!(env, merchant create Grognon)
        .success()
        .stdout(str::is_empty());

    cmd!(env, merchant list)
        .success()
        .stdout(str::contains("1  | Chariot"))
        .stdout(str::contains("2  | Grognon"));

    Ok(())
}

#[test]
fn show() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, merchant show)
        .failure()
        .stderr(str::contains("  <ID>"));

    cmd!(env, merchant show 1)
        .failure()
        .stderr(str::contains("Not found"));

    cmd!(env, merchant create Chariot)
        .success()
        .stdout(str::is_empty());

    cmd!(env, merchant show 1)
        .success()
        .stdout(str::contains("1  | Chariot"));

    Ok(())
}

#[test]
fn create() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, merchant create)
        .failure()
        .stderr(str::contains("  <NAME>"));

    cmd!(env, merchant create Chariot)
        .success()
        .stdout(str::is_empty());

    Ok(())
}

#[test]
fn update() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, merchant update)
        .failure()
        .stderr(str::contains("  <ID>"));

    cmd!(env, merchant update 1)
        .failure()
        .stderr(str::contains("Not found"));

    cmd!(env, merchant create Chariot)
        .success()
        .stdout(str::is_empty());

    cmd!(env, merchant update 1)
        .success()
        .stdout(str::is_empty());

    cmd!(env, merchant show 1)
        .success()
        .stdout(str::contains("1  | Chariot"));

    cmd!(env, merchant update 1 --name Grognon)
        .success()
        .stdout(str::is_empty());

    cmd!(env, merchant show 1)
        .success()
        .stdout(str::contains("1  | Grognon"));

    Ok(())
}

#[test]
fn delete() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, merchant delete)
        .failure()
        .stderr(str::contains("  <ID>"));

    cmd!(env, merchant delete 1)
        .failure()
        .stderr(str::contains("Not found"));

    cmd!(env, merchant create Chariot)
        .success()
        .stdout(str::is_empty());

    cmd!(env, merchant delete 1)
        .failure()
        .stderr(str::contains("confirmation flag"));

    cmd!(env, merchant delete 1 --confirm)
        .success()
        .stdout(str::is_empty());

    cmd!(env, merchant show 1)
        .failure()
        .stderr(str::contains("Not found"));

    Ok(())
}
