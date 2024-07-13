use oxydized_money::CurrencyError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Not found")]
    NotFound,
    #[error("Conflict with existing data. {0}")]
    NonUnique(String),
    #[error("Invalid. {0}")]
    Invalid(String),
    #[error("Parsing version information")]
    VersionError(#[from] semver::Error),
    #[error("Reading currency. {0}")]
    CurrencyError(#[from] CurrencyError),
    #[error("Generic error. {0}")]
    GenericError(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error("Connection error")]
    ConnectionError(#[from] diesel::result::ConnectionError),
    #[error("Diesel error. {0}")]
    DieselError(diesel::result::Error),
}

impl From<diesel::result::Error> for Error {
    fn from(e: diesel::result::Error) -> Error {
        use diesel::result::{
            DatabaseErrorKind,
            Error::{DatabaseError, NotFound},
        };

        match e {
            NotFound => Error::NotFound,
            DatabaseError(DatabaseErrorKind::UniqueViolation, e) => {
                Error::NonUnique(e.message().to_string())
            }
            _ => Error::DieselError(e),
        }
    }
}

pub trait OptionalExtension<T> {
    fn optional(self) -> Result<Option<T>>;
    fn optional_empty_changeset(self) -> Result<Option<T>>;
}

impl<T> OptionalExtension<T> for Result<T> {
    fn optional(self) -> Result<Option<T>> {
        match self {
            Ok(value) => Ok(Some(value)),
            Err(Error::NotFound) => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn optional_empty_changeset(self) -> Result<Option<T>> {
        use diesel::result::{EmptyChangeset, Error::QueryBuilderError};

        match self {
            Ok(value) => Ok(Some(value)),
            Err(Error::DieselError(QueryBuilderError(e)))
                if e.is::<EmptyChangeset>() =>
            {
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub struct ParseTypeError(pub &'static str, pub String);

impl std::fmt::Display for ParseTypeError {
    fn fmt(
        &self,
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "Parse Type Error: {} {}", self.0, self.1)
    }
}
