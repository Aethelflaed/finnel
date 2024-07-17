use crate::common::prelude::*;

#[test]
fn required_arguments() -> Result<()> {
    let env = crate::Env::new()?;

    cmd!(env, record add)
        .failure()
        .stderr(str::contains("  <AMOUNT>"))
        .stderr(str::contains("  <DETAILS>"));

    cmd!(env, record add 10)
        .failure()
        .stderr(str::contains("  <AMOUNT>").not())
        .stderr(str::contains("  <DETAILS>"));

    cmd!(env, record add 10 bread)
        .failure()
        .stderr(str::contains("Account not provided"));

    cmd!(env, account create Cash).success();

    cmd!(env, record add 10 bread -A Cash)
        .success();

    Ok(())
}

#[test]
fn operations() -> Result<()> {
    let env = crate::Env::new()?;
    crate::setup(&env)?;

    cmd!(env, record add 10 bread)
        .success();

    Ok(())
}
