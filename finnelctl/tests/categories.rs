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

    cmd!(env, category list update --create_parent Establishment).success();

    raw_cmd!(env, category list --name Bar delete --confirm)
        .write_stdin("yes")
        .assert()
        .success();

    cmd!(env, category list)
        .success()
        .stdout(str::contains("1  | Bar").not())
        .stdout(str::contains("2  | Restaurant"))
        .stdout(str::contains("3  | Establishment"));

    Ok(())
}

#[test]
fn list_not_in() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, category create Bar).success();
    cmd!(env, category create Restaurant).success();

    cmd!(env, category list)
        .success()
        .stdout(str::contains("1  | Bar"))
        .stdout(str::contains("2  | Restaurant"));

    cmd!(env, report create Report).success();
    cmd!(env, report show Report add Bar).success();

    cmd!(env, category list "--not-in" 1)
        .success()
        .stdout(str::contains("1  | Bar").not())
        .stdout(str::contains("2  | Restaurant"));

    Ok(())
}

#[test]
fn show() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, category show)
        .failure()
        .stderr(str::contains("  <NAME_OR_ID>"));

    cmd!(env, category show Bar)
        .failure()
        .stderr(str::contains("Category not found by name"));

    cmd!(env, category create Bar).success();
    cmd!(env, category show Bar)
        .success()
        .stdout(str::contains("1 | Bar"))
        .stdout(str::contains("\n  Parent:").not())
        .stdout(str::contains("\n  Replaced by:").not());

    cmd!(env, category show 1)
        .success()
        .stdout(str::contains("1 | Bar"));

    cmd!(env, category create Bars).success();
    cmd!(env, category show Bars update --replace_by Bar).success();
    cmd!(env, category show Bars)
        .success()
        .stdout(str::contains("  Replaced by: 1 | Bar"));

    cmd!(env, category create Rent --create_parent Lodging).success();
    cmd!(env, category show Rent)
        .success()
        .stdout(str::contains("  Parent: 3 | Lodging"));

    raw_cmd!(env, category show Rent delete --confirm)
        .write_stdin("yes")
        .assert()
        .success();
    cmd!(env, category show Rent).failure();

    Ok(())
}

#[test]
fn show_records() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, category create Bar).success();
    cmd!(env, account create Cash).success();
    cmd!(env, account create Bank).success();

    cmd!(env, category show Bar -A Bank)
        .success()
        .stdout(str::contains("No associated records"));

    cmd!(env, record create -A Cash 5 beer --category Bar).success();
    cmd!(env, record create -A Bank 10 beer --category Bar).success();

    cmd!(env, category show Bar)
        .success()
        .stdout(str::contains("€ -5.00"))
        .stdout(str::contains("€ -10.00"));

    Ok(())
}

#[test]
fn show_records_from_children_but_not_parents() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, category create Bar "--create-parent" Alcohol).success();
    cmd!(env, account create Cash).success();
    cmd!(env, record create -A Cash 5 beer --category Bar).success();
    cmd!(env, record create -A Cash 10 wine --category Alcohol).success();

    cmd!(env, category show Bar)
        .success()
        .stdout(str::contains("beer"))
        .stdout(str::contains("wine").not());

    cmd!(env, category show Alcohol)
        .success()
        .stdout(str::contains("beer"))
        .stdout(str::contains("wine"));

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
        .stderr(str::contains("  <NAME_OR_ID>"));

    cmd!(env, category update Bar)
        .failure()
        .stderr(str::contains("Category not found by name"));

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
        .stderr(str::contains("Category not found by name"));

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
        .stderr(str::contains("  <NAME_OR_ID>"));

    cmd!(env, category delete Bar)
        .failure()
        .stderr(str::contains("Category not found by name"));

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
        .stderr(str::contains("Category not found by name"));

    Ok(())
}
