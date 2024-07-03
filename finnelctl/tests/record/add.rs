use crate::common::prelude::*;

#[test]
fn required_arguments() -> Result<()> {
    let env = crate::Env::new()?;

    env.command()?
        .arg("record")
        .arg("add")
        .assert()
        .failure()
        .stderr(str::contains("  <AMOUNT>"))
        .stderr(str::contains("  <DETAILS>"));

    env.command()?
        .arg("record")
        .arg("add")
        .arg("10")
        .assert()
        .failure()
        .stderr(str::contains("  <AMOUNT>").not())
        .stderr(str::contains("  <DETAILS>"));

    env.command()?
        .arg("record")
        .arg("add")
        .arg("10")
        .arg("bread")
        .assert()
        .failure()
        .stderr(str::contains("Account not provided"));

    env.command()?
        .arg("account")
        .arg("create")
        .arg("Cash")
        .assert()
        .success()
        .stdout(str::is_empty());

    env.command()?
        .arg("record")
        .arg("add")
        .arg("10")
        .arg("bread")
        .arg("-A")
        .arg("Cash")
        .assert()
        .success();

    Ok(())
}

#[test]
fn operations() -> Result<()> {
    let env = crate::Env::new()?;
    crate::setup(&env)?;

    env.command()?
        .arg("record")
        .arg("add")
        .arg("10")
        .arg("bread")
        .assert()
        .success();

    Ok(())
}
