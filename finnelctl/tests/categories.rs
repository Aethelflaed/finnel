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

    cmd!(env, category create Bar).success();
    cmd!(env, category create Restaurant).success();

    cmd!(env, category list)
        .success()
        .stdout(str::contains("1  | Bar"))
        .stdout(str::contains("2  | Restaurant"));

    cmd!(env, category list update --create_parent "Bar or Restaurant").success();

    Ok(())
}

#[test]
fn show() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, category show)
        .failure()
        .stderr(str::contains("  <NAME>"));

    cmd!(env, category show Bar)
        .failure()
        .stderr(str::contains("Not found"));

    cmd!(env, category create Bar).success();
    cmd!(env, category show Bar)
        .success()
        .stdout(str::contains("1 | Bar"))
        .stdout(str::contains("\n  Parent:").not())
        .stdout(str::contains("\n  Replaced by:").not())
        .stdout(str::contains("Specify an account"));

    cmd!(env, account create Cash).success();
    cmd!(env, category show Bar -A Cash)
        .success()
        .stdout(str::contains("No associated"));

    cmd!(env, record create -A Cash 5 beer --category Bar).success();
    cmd!(env, category show Bar -A Cash)
        .success()
        .stdout(str::contains("€ -5.00"));

    cmd!(env, category create Bars --replace_by Bar).success();
    cmd!(env, category show Bars)
        .success()
        .stdout(str::contains("  Replaced by: 1 | Bar"));

    cmd!(env, category create Rent --create_parent Lodging).success();
    cmd!(env, category show Rent)
        .success()
        .stdout(str::contains("  Parent: 3 | Lodging"));

    Ok(())
}

#[test]
fn create() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, category create)
        .failure()
        .stderr(str::contains("  <NAME>"));

    cmd!(env, category create Bar)
        .success()
        .stdout(str::is_empty());

    Ok(())
}

#[test]
fn update() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, category update)
        .failure()
        .stderr(str::contains("  <NAME>"));

    cmd!(env, category update Bar)
        .failure()
        .stderr(str::contains("Not found"));

    cmd!(env, category create Bar).success();

    cmd!(env, category update Bar)
        .success()
        .stdout(str::is_empty());

    cmd!(env, category show Bar)
        .success()
        .stdout(str::contains("1 | Bar"));

    cmd!(env, category update Bar --new_name Resto)
        .success()
        .stdout(str::is_empty());

    cmd!(env, category show Bar)
        .failure()
        .stderr(str::contains("Not found"));

    cmd!(env, category show Resto)
        .success()
        .stdout(str::contains("1 | Resto"));

    Ok(())
}

#[test]
fn delete() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, category delete)
        .failure()
        .stderr(str::contains("  <NAME>"));

    cmd!(env, category delete Bar)
        .failure()
        .stderr(str::contains("Not found"));

    cmd!(env, category create Bar).success();

    cmd!(env, category delete Bar)
        .failure()
        .stderr(str::contains("requires confirmation"));

    raw_cmd!(env, category delete Bar --confirm)
        .write_stdin("yes")
        .assert()
        .success()
        .stdout(str::contains("you really want"));

    cmd!(env, category show Bar)
        .failure()
        .stderr(str::contains("Not found"));

    Ok(())
}
