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

    cmd!(env, merchant create Chariot).success();
    cmd!(env, merchant create Grognon).success();

    cmd!(env, merchant list)
        .success()
        .stdout(str::contains("1  | Chariot"))
        .stdout(str::contains("2  | Grognon"));

    cmd!(env, merchant list update --create_replace_by Bar).success();

    raw_cmd!(env, merchant list --name Grognon delete --confirm)
        .write_stdin("yes")
        .assert()
        .success();

    cmd!(env, merchant list)
        .success()
        .stdout(str::contains("1  | Chariot"))
        .stdout(str::contains("2  | Grognon").not())
        .stdout(str::contains("3  | Bar"));

    Ok(())
}

#[test]
fn show() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, merchant show)
        .failure()
        .stderr(str::contains("  <NAME_OR_ID>"));

    cmd!(env, merchant show Chariot)
        .failure()
        .stderr(str::contains("Merchant not found by name"));

    cmd!(env, merchant create Chariot).success();
    cmd!(env, merchant show Chariot)
        .success()
        .stdout(str::contains("1 | Chariot"))
        .stdout(str::contains("Default category").not());

    cmd!(env, merchant show 1)
        .success()
        .stdout(str::contains("1 | Chariot"));

    cmd!(env, merchant show Chariot update --create_default_category Bar).success();

    cmd!(env, merchant show Chariot)
        .success()
        .stdout(str::contains("1 | Chariot"))
        .stdout(str::contains("Default category: 1 | Bar"));

    raw_cmd!(env, merchant show Chariot delete --confirm)
        .write_stdin("yes")
        .assert()
        .success();
    cmd!(env, merchant show Chariot).failure();

    Ok(())
}

#[test]
fn show_records() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, merchant create Chariot).success();
    cmd!(env, account create Cash).success();
    cmd!(env, account create Bank).success();

    cmd!(env, merchant show Chariot -A Bank)
        .success()
        .stdout(str::contains("No associated records for account Cash").not())
        .stdout(str::contains("No associated records for account Bank"));

    cmd!(env, merchant show Chariot -A Cash)
        .success()
        .stdout(str::contains("No associated records for account Cash"))
        .stdout(str::contains("No associated records for account Bank").not());

    cmd!(env, record create -A Cash 5 beer --merchant Chariot).success();

    cmd!(env, merchant show Chariot -A Cash)
        .success()
        .stdout(str::contains("No associated records for account Cash").not())
        .stdout(str::contains("€ -5.00"));

    cmd!(env, merchant show Chariot)
        .success()
        .stdout(str::contains("No associated records for account Cash").not())
        .stdout(str::contains("No associated records for account Bank"))
        .stdout(str::contains("€ -5.00"));

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

    cmd!(env, merchant create Chariot)
        .failure()
        .stderr(str::contains("Conflict with existing data"));

    cmd!(env, merchant create Grognon --create_default_category Bar)
        .success()
        .stdout(str::is_empty());

    cmd!(env, merchant show Grognon)
        .success()
        .stdout(str::contains("Default category: 1 | Bar"));

    cmd!(env, merchant create Grochion --default_category Bar).success();
    cmd!(env, merchant show Grochion)
        .success()
        .stdout(str::contains("Default category: 1 | Bar"));

    cmd!(env, merchant create Uraidla --default_category_id 1).success();
    cmd!(env, merchant show Uraidla)
        .success()
        .stdout(str::contains("Default category: 1 | Bar"));

    Ok(())
}

#[test]
fn update() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, merchant update)
        .failure()
        .stderr(str::contains("  <NAME_OR_ID>"));

    cmd!(env, merchant update Chariot)
        .failure()
        .stderr(str::contains("Merchant not found by name"));

    cmd!(env, merchant create Chariot)
        .success()
        .stdout(str::is_empty());

    cmd!(env, merchant update Chariot)
        .success()
        .stdout(str::is_empty());

    cmd!(env, merchant show Chariot)
        .success()
        .stdout(str::contains("1 | Chariot"))
        .stdout(str::contains("  Default category:").not())
        .stdout(str::contains("  Replaced by:").not());

    cmd!(env, merchant update Chariot --new_name Grognon)
        .success()
        .stdout(str::is_empty());

    cmd!(env, merchant show Chariot)
        .failure()
        .stderr(str::contains("Merchant not found by name"));

    cmd!(env, merchant show Grognon)
        .success()
        .stdout(str::contains("1 | Grognon"));

    cmd!(env, category create Restaurant).success();
    cmd!(env, category create Bar).success();

    cmd!(env, merchant update Grognon --default_category Restaurant).success();
    cmd!(env, merchant show Grognon)
        .success()
        .stdout(str::contains("  Default category: 1 | Restaurant"));

    cmd!(env, merchant update Grognon --default_category_id 2).success();
    cmd!(env, merchant show Grognon)
        .success()
        .stdout(str::contains("  Default category: 2 | Bar"));

    cmd!(env, merchant update Grognon --no_default_category).success();
    cmd!(env, merchant show Grognon)
        .success()
        .stdout(str::contains("  Default category:").not());

    cmd!(env, merchant create LeGrognon --replace_by Grognon).success();
    cmd!(env, merchant show LeGrognon)
        .success()
        .stdout(str::contains("  Replaced by: 1 | Grognon"));

    Ok(())
}

#[test]
fn delete() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, merchant delete)
        .failure()
        .stderr(str::contains("  <NAME_OR_ID>"));

    cmd!(env, merchant delete Chariot)
        .failure()
        .stderr(str::contains("Merchant not found by name"));

    cmd!(env, merchant create Chariot)
        .success()
        .stdout(str::is_empty());

    cmd!(env, merchant delete Chariot)
        .failure()
        .stderr(str::contains("requires confirmation"));

    raw_cmd!(env, merchant delete Chariot --confirm)
        .write_stdin("yes")
        .assert()
        .success()
        .stdout(str::contains("you really want"));

    cmd!(env, merchant show Chariot)
        .failure()
        .stderr(str::contains("Merchant not found by name"));

    Ok(())
}
