use crate::common::prelude::*;

pub fn setup(env: &crate::Env) -> Result<()> {
    cmd!(env, account create Cash).success();
    cmd!(env, account create Bank).success();

    cmd!(env, category create beer).success();
    cmd!(env, category create food).success();
    cmd!(env, merchant create grocer).success();
    cmd!(env, record create 10 Bread
        --account Cash
        --category food
        --merchant grocer
        "--value-date" "2024-08-01"
        "--operation-date" "2024-08-10"
    )
    .success();
    cmd!(env, record create 5 Beer
        --account Bank
        --category beer
        "--value-date" "2024-08-10"
        "--operation-date" "2024-08-01"
    )
    .success();

    Ok(())
}

#[test]
fn empty() -> Result<()> {
    let env = crate::Env::new()?;

    cmd!(env, record list).success().stdout(str::is_empty());

    Ok(())
}

#[test]
fn all() -> Result<()> {
    let env = crate::Env::new()?;
    setup(&env)?;

    let stdout = cmd!(env, record list)
        .success()
        .into_stdout();

    let mut start = 0;
    for pattern in ["Bread", "Beer"] {
        if let Some(index) = stdout[start..].find(pattern) {
            start += index;
        } else {
            assert!(false, "Unable to find {} in {}", pattern, &stdout[start..]);
        }
    }

    Ok(())
}

#[test]
fn sort_by_date() -> Result<()> {
    let env = crate::Env::new()?;
    setup(&env)?;

    let stdout = cmd!(env, record list --sort date).success().into_stdout();

    let mut start = 0;
    for pattern in ["Bread", "Beer"] {
        if let Some(index) = stdout[start..].find(pattern) {
            start += index;
        } else {
            assert!(false, "Unable to find {} in {}", pattern, &stdout[start..]);
        }
    }

    let stdout = cmd!(env, record list --sort "date.desc").success().into_stdout();

    let mut start = 0;
    for pattern in ["Beer", "Bread"] {
        if let Some(index) = stdout[start..].find(pattern) {
            start += index;
        } else {
            assert!(false, "Unable to find {} in {}", pattern, &stdout[start..]);
        }
    }

    let stdout = cmd!(env, record list --sort "date.desc" "--operation-date").success().into_stdout();

    let mut start = 0;
    for pattern in ["Bread", "Beer"] {
        if let Some(index) = stdout[start..].find(pattern) {
            start += index;
        } else {
            assert!(false, "Unable to find {} in {}", pattern, &stdout[start..]);
        }
    }

    Ok(())
}

#[test]
fn filter_by_account() -> Result<()> {
    let env = crate::Env::new()?;
    setup(&env)?;

    cmd!(env, record list --account Cash)
        .success()
        .stdout(str::contains("Bread"))
        .stdout(str::contains("Beer").not());

    Ok(())
}

#[test]
fn filter_by_category() -> Result<()> {
    let env = crate::Env::new()?;
    setup(&env)?;

    cmd!(env, record list --category beer)
        .success()
        .stdout(str::contains("Beer"))
        .stdout(str::contains("Bread").not());

    Ok(())
}

#[test]
fn filter_by_merchant() -> Result<()> {
    let env = crate::Env::new()?;
    setup(&env)?;

    cmd!(env, record list --merchant grocer)
        .success()
        .stdout(str::contains("Bread"))
        .stdout(str::contains("Beer").not());

    cmd!(env, record list "--no-merchant")
        .success()
        .stdout(str::contains("Bread").not())
        .stdout(str::contains("Beer"));

    Ok(())
}

#[test]
fn filter_from_is_inclusive() -> Result<()> {
    let env = crate::Env::new()?;
    setup(&env)?;

    cmd!(env, record list --from "2024-08-01")
        .success()
        .stdout(str::contains("Bread"))
        .stdout(str::contains("Beer"));

    Ok(())
}

#[test]
fn filter_from() -> Result<()> {
    let env = crate::Env::new()?;
    setup(&env)?;

    cmd!(env, record list --from "2024-08-02")
        .success()
        .stdout(str::contains("Bread").not())
        .stdout(str::contains("Beer"));

    cmd!(env, record list --from "2024-08-02" "--operation-date")
        .success()
        .stdout(str::contains("Bread"))
        .stdout(str::contains("Beer").not());

    Ok(())
}

#[test]
fn filter_to_is_exclusive() -> Result<()> {
    let env = crate::Env::new()?;
    setup(&env)?;

    cmd!(env, record list --to "2024-08-01")
        .success()
        .stdout(str::is_empty());

    Ok(())
}

#[test]
fn filter_to() -> Result<()> {
    let env = crate::Env::new()?;
    setup(&env)?;

    cmd!(env, record list --to "2024-08-02")
        .success()
        .stdout(str::contains("Bread"))
        .stdout(str::contains("Beer").not());

    cmd!(env, record list --to "2024-08-02" "--operation-date")
        .success()
        .stdout(str::contains("Bread").not())
        .stdout(str::contains("Beer"));

    Ok(())
}

#[test]
fn filter_greater_than_is_inclusive() -> Result<()> {
    let env = crate::Env::new()?;
    setup(&env)?;

    cmd!(env, record list "--greater-than" "5")
        .success()
        .stdout(str::contains("Bread"))
        .stdout(str::contains("Beer"));

    Ok(())
}

#[test]
fn filter_greater_than() -> Result<()> {
    let env = crate::Env::new()?;
    setup(&env)?;

    cmd!(env, record list "--greater-than" "6")
        .success()
        .stdout(str::contains("Bread"))
        .stdout(str::contains("Beer").not());

    Ok(())
}

#[test]
fn filter_less_than_is_exclusive() -> Result<()> {
    let env = crate::Env::new()?;
    setup(&env)?;

    cmd!(env, record list "--less-than" "5")
        .success()
        .stdout(str::is_empty());

    Ok(())
}

#[test]
fn filter_less_than() -> Result<()> {
    let env = crate::Env::new()?;
    setup(&env)?;

    cmd!(env, record list "--less-than" "6")
        .success()
        .stdout(str::contains("Bread").not())
        .stdout(str::contains("Beer"));

    Ok(())
}
