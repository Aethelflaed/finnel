use oxydized_money::CurrencyError;
use std::array::TryFromSliceError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
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
    #[error("Generic error")]
    GenericError(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("Connection error")]
    ConnectionError(#[from] diesel::prelude::ConnectionError),
    #[error("Diesel error")]
    DieselError(diesel::result::Error),
}

impl From<diesel::result::Error> for Error {
    fn from(e: diesel::result::Error) -> Error {
        match e {
            diesel::result::Error::NotFound => Error::NotFound,
            _ => Error::DieselError(e),
        }
    }
}

pub trait OptionalExtension<T> {
    fn optional(self) -> Result<Option<T>>;
}

impl<T> OptionalExtension<T> for Result<T> {
    fn optional(self) -> Result<Option<T>> {
        match self {
            Ok(value) => Ok(Some(value)),
            Err(Error::NotFound) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
