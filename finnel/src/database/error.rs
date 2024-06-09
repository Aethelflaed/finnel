use crate::transaction;
use oxydized_money::CurrencyError;
use std::array::TryFromSliceError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Sqlite error")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Not found")]
    NotFound,
    #[error("Not persisted")]
    NotPersisted,
    #[error("Parsing date error")]
    DateParseError(#[from] chrono::ParseError),
    #[error("Parsing transaction type error")]
    TransactionTypeParseError(#[from] transaction::ParseTypeError),
    #[error("Parsing version information")]
    VersionError(#[from] semver::Error),
    #[error("Reading decimal")]
    TryFromSliceError(#[from] TryFromSliceError),
    #[error("Reading currency")]
    CurrencyError(#[from] CurrencyError),
}
