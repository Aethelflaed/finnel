#[macro_use]
mod common;
use common::prelude::*;

#[test]
fn empty() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, category)
        .failure()
        .stderr(str::contains("Usage:"));

    Ok(())
}

#[test]
fn list() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, category create Chariot)
        .success()
        .stdout(str::is_empty());

    cmd!(env, category create Grognon)
        .success()
        .stdout(str::is_empty());

    cmd!(env, category list)
        .success()
        .stdout(str::contains("1  | Chariot"))
        .stdout(str::contains("2  | Grognon"));

    Ok(())
}

#[test]
fn show() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, category show)
        .failure()
        .stderr(str::contains("  <ID>"));

    cmd!(env, category show 1)
        .failure()
        .stderr(str::contains("Not found"));

    cmd!(env, category create Chariot)
        .success()
        .stdout(str::is_empty());

    cmd!(env, category show 1)
        .success()
        .stdout(str::contains("1  | Chariot"));

    Ok(())
}

#[test]
fn create() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, category create)
        .failure()
        .stderr(str::contains("  <NAME>"));

    cmd!(env, category create Chariot)
        .success()
        .stdout(str::is_empty());

    Ok(())
}

#[test]
fn update() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, category update)
        .failure()
        .stderr(str::contains("  <ID>"));

    cmd!(env, category update 1)
        .failure()
        .stderr(str::contains("Not found"));

    cmd!(env, category create Chariot)
        .success()
        .stdout(str::is_empty());

    cmd!(env, category update 1)
        .success()
        .stdout(str::is_empty());

    cmd!(env, category show 1)
        .success()
        .stdout(str::contains("1  | Chariot"));

    cmd!(env, category update 1 --name Grognon)
        .success()
        .stdout(str::is_empty());

    cmd!(env, category show 1)
        .success()
        .stdout(str::contains("1  | Grognon"));

    Ok(())
}

#[test]
fn delete() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, category delete)
        .failure()
        .stderr(str::contains("  <ID>"));

    cmd!(env, category delete 1)
        .failure()
        .stderr(str::contains("Not found"));

    cmd!(env, category create Chariot)
        .success()
        .stdout(str::is_empty());

    cmd!(env, category delete 1)
        .failure()
        .stderr(str::contains("confirmation flag"));

    cmd!(env, category delete 1 --confirm)
        .success()
        .stdout(str::is_empty());

    cmd!(env, category show 1)
        .failure()
        .stderr(str::contains("Not found"));

    Ok(())
}
