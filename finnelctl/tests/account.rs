mod common;
use common::prelude::*;

#[test]
fn operations() -> Result<()> {
    let env = Env::new()?;

    env.command()?
        .arg("account")
        .arg("create")
        .arg("Cash")
        .assert()
        .success()
        .stdout(str::is_empty());

    env.command()?
        .arg("account")
        .arg("list")
        .assert()
        .success()
        .stdout(str::contains("Cash"));

    env.command()?
        .arg("account")
        .arg("show")
        .assert()
        .failure()
        .stderr(str::contains("Account not provided"));

    env.command()?
        .arg("-A")
        .arg("Cash")
        .arg("account")
        .arg("show")
        .assert()
        .success()
        .stdout(str::contains("EUR 0"));

    env.command()?
        .arg("account")
        .arg("default")
        .assert()
        .success()
        .stdout(str::contains("<not set>"));

    env.command()?
        .arg("account")
        .arg("-A")
        .arg("Cash")
        .arg("default")
        .assert()
        .success()
        .stdout(str::is_empty());

    env.command()?
        .arg("account")
        .arg("default")
        .assert()
        .success()
        .stdout(str::contains("Cash"));

    env.command()?
        .arg("account")
        .arg("show")
        .assert()
        .success()
        .stdout(str::contains("EUR 0"));

    env.command()?
        .arg("account")
        .arg("delete")
        .assert()
        .failure()
        .stderr(str::contains("confirmation"));

    env.command()?
        .arg("account")
        .arg("delete")
        .arg("-A")
        .arg("Cash")
        .assert()
        .failure()
        .stderr(str::contains("confirmation"));

    env.command()?
        .arg("-A")
        .arg("Cash")
        .arg("account")
        .arg("delete")
        .arg("--confirm")
        .assert()
        .success()
        .stdout(str::is_empty());

    env.command()?
        .arg("account")
        .arg("default")
        .assert()
        .success()
        .stdout(str::contains("<not set>"));

    Ok(())
}
