#[macro_use]
mod common;
use common::prelude::*;

#[test]
fn empty() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, report).failure().stderr(str::contains("Usage:"));

    Ok(())
}

#[test]
fn show() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, report create Foo).success();
    cmd!(env, category create Bar).success();
    cmd!(env, category create Restaurant).success();

    cmd!(env, report show 1)
        .success()
        .stdout(str::contains("1 | Foo"))
        .stdout(str::contains("Bar").not())
        .stdout(str::contains("Restaurant").not());

    cmd!(env, report show Foo add Bar 2)
        .success()
        .stdout(str::is_empty());

    cmd!(env, report show 1)
        .success()
        .stdout(str::contains("1 | Foo"))
        .stdout(str::contains("Bar"))
        .stdout(str::contains("Restaurant"));

    cmd!(env, report show Foo remove 1)
        .success()
        .stdout(str::is_empty());

    cmd!(env, report show 1)
        .success()
        .stdout(str::contains("1 | Foo"))
        .stdout(str::contains("Bar").not())
        .stdout(str::contains("Restaurant"));

    Ok(())
}

#[test]
fn list() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, report create Foo).success();
    cmd!(env, report create Bar).success();

    cmd!(env, report list)
        .success()
        .stdout(str::contains("1  | Foo"))
        .stdout(str::contains("2  | Bar"));

    Ok(())
}

#[test]
fn create() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, report create Foo)
        .success()
        .stdout(str::is_empty());
    cmd!(env, report create Foo)
        .failure()
        .stderr(str::contains("Conflict"));

    Ok(())
}

#[test]
fn delete() -> Result<()> {
    let env = Env::new()?;

    cmd!(env, report delete Foo)
        .failure()
        .stderr(str::contains("not found"));

    cmd!(env, report create Foo).success();
    cmd!(env, report create Bar).success();

    cmd!(env, report delete Foo)
        .failure()
        .stderr(str::contains("requires confirmation"));

    cmd!(env, report delete Foo --confirm)
        .failure()
        .stdout("Do you really want to do that?\n")
        .stderr(str::contains("requires confirmation"));

    raw_cmd!(env, report delete Foo --confirm)
        .write_stdin("yes")
        .assert()
        .success()
        .stdout("Do you really want to do that?\n");

    raw_cmd!(env, report delete 2 --confirm)
        .write_stdin("yes")
        .assert()
        .success()
        .stdout("Do you really want to do that?\n");

    Ok(())
}
