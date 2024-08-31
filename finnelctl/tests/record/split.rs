use crate::common::prelude::*;

pub fn setup(env: &crate::Env) -> Result<()> {
    crate::setup(env)?;

    cmd!(env, category create beer --create_replace_by Beer).success();
    cmd!(env, category create food --create_replace_by Food).success();
    cmd!(env, merchant create grocer --create_replace_by Grocer).success();
    cmd!(env, record create 10 Bread --category food --merchant grocer).success();

    Ok(())
}

#[test]
fn required_arguments() -> Result<()> {
    let env = crate::Env::new()?;
    setup(&env)?;

    cmd!(env, record show 1 split)
        .failure()
        .stderr(str::contains("\n  <AMOUNT>"));

    cmd!(env, record show 1 split 5).success();

    Ok(())
}

#[test]
fn operations() -> Result<()> {
    let env = crate::Env::new()?;
    setup(&env)?;

    cmd!(env, record show 1 split 5)
        .success()
        .stdout(str::is_empty());

    cmd!(env, record show 2)
        .success()
        .stdout(str::contains("Bread"))
        .stdout(str::contains("Food"))
        .stdout(str::contains("Grocer"))
        .stdout(str::contains("€ -5.00"));

    cmd!(env, record show 2 split 1 --details Candy)
        .success()
        .stdout(str::is_empty());

    cmd!(env, record show 3)
        .success()
        .stdout(str::contains("Candy"))
        .stdout(str::contains("Food"))
        .stdout(str::contains("Grocer"))
        .stdout(str::contains("€ -1.00"));

    cmd!(env, record show 1 split 2 --category beer)
        .success()
        .stdout(str::is_empty());

    cmd!(env, record show 4)
        .success()
        .stdout(str::contains("Beer"))
        .stdout(str::contains("€ -2.00"));

    cmd!(env, record show 1)
        .success()
        .stdout(str::contains("€ -3.00"));

    Ok(())
}
