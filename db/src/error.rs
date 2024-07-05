use oxydized_money::CurrencyError;
use std::array::TryFromSliceError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Sqlite error")]
    Sqlite(rusqlite::Error),
    #[error("Not found")]
    NotFound,
    #[error("Not persisted")]
    NotPersisted,
    #[error("Conflict with existing data. {0}")]
    NonUnique(String),
    #[error("Invalid")]
    Invalid(String),
    #[error("Parsing version information")]
    VersionError(#[from] semver::Error),
    #[error("Reading decimal")]
    TryFromSliceError(#[from] TryFromSliceError),
    #[error("Reading currency")]
    CurrencyError(#[from] CurrencyError),
}

impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        match e {
            rusqlite::Error::SqliteFailure(e, msg)
                if e.extended_code == 2067 =>
            {
                Error::NonUnique(msg.unwrap_or("".to_string()))
            }
            _ => Error::Sqlite(e),
        }
    }
}
