use anyhow::Result;

#[test]
fn operations() -> Result<()> {
    let env = crate::Env::new()?;

    env.command()?
        .arg("account")
        .arg("create")
        .arg("Cash")
        .assert()
        .success()
        .stdout(predicates::str::is_empty());

    env.command()?
        .arg("account")
        .arg("list")
        .assert()
        .success()
        .stdout(predicates::str::contains("Cash"));

    env.command()?
        .arg("account")
        .arg("default")
        .assert()
        .success()
        .stdout(predicates::str::contains("<not set>"));

    env.command()?
        .arg("account")
        .arg("default")
        .arg("Cash")
        .assert()
        .success()
        .stdout(predicates::str::is_empty());

    env.command()?
        .arg("account")
        .arg("default")
        .assert()
        .success()
        .stdout(predicates::str::contains("Cash"));

    env.command()?
        .arg("account")
        .arg("delete")
        .arg("Cash")
        .assert()
        .failure()
        .stderr(predicates::str::contains("confirmation"));

    env.command()?
        .arg("account")
        .arg("delete")
        .arg("Cash")
        .arg("--confirm")
        .assert()
        .success()
        .stdout(predicates::str::is_empty());

    env.command()?
        .arg("account")
        .arg("default")
        .assert()
        .success()
        .stdout(predicates::str::contains("<not set>"));

    Ok(())
}
