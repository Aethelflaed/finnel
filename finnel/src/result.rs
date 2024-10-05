use oxydized_money::CurrencyError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(derive_more::Error, derive_more::Display, derive_more::From, Debug)]
pub enum Error {
    #[display("Not found")]
    NotFound,
    #[display("{_0} not found")]
    ModelNotFound(#[error(not(source))] &'static str),
    #[display("{_0} not found by {_1}")]
    ModelNotFoundBy(&'static str, &'static str),
    #[display("Conflict with existing data. {_0}")]
    NonUnique(#[error(not(source))] String),
    #[display("Invalid. {_0}")]
    Invalid(#[error(not(source))] String),
    #[display("Parsing version information")]
    #[from]
    VersionError(semver::Error),
    #[display("Reading currency. {_0}")]
    #[from]
    CurrencyError(CurrencyError),
    #[display("Generic error. {_0}")]
    #[from]
    GenericError(Box<dyn std::error::Error + Send + Sync>),
    #[display("Connection error")]
    #[from]
    ConnectionError(diesel::result::ConnectionError),
    #[display("Diesel error. {_0}")]
    DieselError(diesel::result::Error),
    #[display("Invalid month {_0}/{_1}")]
    InvalidMonth(i32, i32),
    #[display("Invalid week {_0:?}/{_1}")]
    InvalidWeek(chrono::IsoWeek, chrono::Weekday),
}

impl Error {
    pub fn from_diesel_error(
        error: diesel::result::Error,
        model: &'static str,
        by: Option<&'static str>,
    ) -> Self {
        match error {
            diesel::result::Error::NotFound => {
                if let Some(by) = by {
                    Error::ModelNotFoundBy(model, by)
                } else {
                    Error::ModelNotFound(model)
                }
            }
            _ => error.into(),
        }
    }

    pub fn is_not_found(&self) -> bool {
        matches!(
            self,
            Error::NotFound | Error::ModelNotFound(_) | Error::ModelNotFoundBy(_, _)
        )
    }
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
            Err(Error::DieselError(QueryBuilderError(e))) if e.is::<EmptyChangeset>() => Ok(None),
            Err(e) => Err(e),
        }
    }
}

#[derive(Debug, PartialEq, Eq, derive_more::Display, derive_more::Error)]
#[display("Parse Type Error: {_0} {_1}")]
pub struct ParseTypeError(pub &'static str, pub String);
