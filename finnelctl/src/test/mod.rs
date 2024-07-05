#![cfg(test)]

pub mod prelude {
    pub use crate::test::{self, with_dirs};
    pub use pretty_assertions::assert_eq;
    pub use assert_fs::fixture::{FileWriteStr, PathChild};
}


pub fn with_temp_dir<F, R>(function: F) -> R
where
    F: FnOnce(&assert_fs::TempDir) -> R,
{
    let temp = assert_fs::TempDir::new().unwrap();
    let result = function(&temp);

    // The descrutor would silence any issue, so we call close() explicitly
    temp.close().unwrap();

    result
}

pub fn with_config_dir<F, R>(function: F) -> R
where
    F: FnOnce(&assert_fs::TempDir) -> R,
{
    with_temp_dir(|temp| {
        temp_env::with_var(
            "FINNEL_CONFIG",
            Some(temp.path().as_os_str()),
            || function(&temp),
        )
    })
}

pub fn with_data_dir<F, R>(function: F) -> R
where
    F: FnOnce(&assert_fs::TempDir) -> R,
{
    with_temp_dir(|temp| {
        temp_env::with_var("FINNEL_DATA", Some(temp.path().as_os_str()), || {
            function(&temp)
        })
    })
}

pub fn with_dirs<F, R>(function: F) -> R
where
    F: FnOnce(&assert_fs::TempDir, &assert_fs::TempDir) -> R,
{
    with_config_dir(|config| with_data_dir(|data| function(&config, &data)))
}
