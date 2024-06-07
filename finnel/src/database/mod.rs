use std::path::Path;

use semver::Version;

use rusqlite::types::{FromSql, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use rusqlite::{Connection};

use oxydized_money::{Amount, Currency, CurrencyError, Decimal};

use crate::transaction;

#[derive(Copy, Clone, Debug, PartialEq, Eq, derive_more::From)]
pub struct Id(i64);

impl FromSql for Id {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        Ok(value.as_i64()?.into())
    }
}

impl ToSql for Id {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        self.0.to_sql()
    }
}

#[derive(Copy, Clone, Debug, derive_more::From, derive_more::Into)]
pub struct Money(Amount);

impl Default for Money {
    fn default() -> Self {
        Self(Amount(Decimal::new(0, 0), Currency::EUR))
    }
}

impl Money {
    fn serialize(&self) -> Vec<u8> {
        let mut vec = Vec::<u8>::with_capacity(18);
        vec.extend_from_slice(&self.0 .0.serialize());
        vec.extend_from_slice(&self.0 .1.numeric().to_be_bytes());
        vec
    }

    fn deserialize(vec: &[u8]) -> Result<Self> {
        Ok(Self(Amount(
            Decimal::deserialize(vec[0..16].try_into()?),
            Currency::from_numeric(u16::from_be_bytes(vec[16..18].try_into()?))
                .ok_or(CurrencyError::Unknown)?,
        )))
    }
}

impl FromSql for Money {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        Self::deserialize(value.as_bytes()?)
            .map_err(|e| rusqlite::types::FromSqlError::Other(Box::new(e)))
    }
}

impl ToSql for Money {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Owned(rusqlite::types::Value::Blob(
            self.serialize(),
        )))
    }
}

pub struct Database {
    pub connection: Connection,
}

impl From<Connection> for Database {
    fn from(connection: Connection) -> Self {
        Database { connection }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Sqlite error")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Not found")]
    NotFound,
    #[error("Parsing date error")]
    DateParseError(#[from] chrono::ParseError),
    #[error("Parsing transaction type error")]
    TransactionTypeParseError(#[from] transaction::ParseTypeError),
    #[error("Parsing version information")]
    VersionError(#[from] semver::Error),
    #[error("Reading decimal")]
    TryFromSliceError(#[from] std::array::TryFromSliceError),
    #[error("Reading currency")]
    CurrencyError(#[from] CurrencyError),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Database {
    pub fn open<T: AsRef<Path>>(path: T) -> Result<Database> {
        match Connection::open(path) {
            Ok(connection) => Ok(Database { connection }),
            Err(e) => Err(e.into()),
        }
    }

    pub fn memory() -> Result<Database> {
        match Connection::open_in_memory() {
            Ok(connection) => Ok(Database { connection }),
            Err(e) => Err(e.into()),
        }
    }

    pub fn version(&self) -> Result<Version> {
        let mut statement = self.connection.prepare(
            "
        SELECT 
            name
        FROM 
            sqlite_schema
        WHERE 
            name = 'finnel' AND
            type ='table' AND 
            name NOT LIKE 'sqlite_%';",
        )?;

        {
            let mut rows = statement.query([])?;

            if rows.next()?.is_none() {
                return Ok(Version::new(0, 0, 0));
            }
        }

        statement = self
            .connection
            .prepare("SELECT value FROM finnel WHERE key = 'version'")?;

        let mut rows = statement.query([])?;

        if let Some(row) = rows.next()? {
            Ok(Version::parse(row.get::<usize, String>(0)?.as_str())?)
        } else {
            Ok(Version::new(0, 0, 0))
        }
    }
}

pub trait Entity: Sized {
    fn id(&self) -> Option<Id>;

    fn find(db: &Database, id: Id) -> Result<Self>;
    fn save(&mut self, db: &Database) -> Result<()>;
}

pub(crate) trait Upgrade {
    fn setup(db: &Database) -> Result<()> {
        let version = db.version()?;
        let current = Version::parse(env!("CARGO_PKG_VERSION"))?;

        if version == Version::new(0, 0, 0) {
            db.connection.execute(
                "
            CREATE TABLE IF NOT EXISTS finnel (
                key TEXT NOT NULL UNIQUE,
                value TEXT
            );
            ",
                (),
            )?;
        }

        if version < current {
            Self::upgrade_from(db, &version)?;
        }

        if version == Version::new(0, 0, 0) {
            db.connection.execute(
                format!(
                    "INSERT INTO finnel (key, value) VALUES('version', '{current}');"
                ).as_str(), ()
            )?;
        } else {
            db.connection.execute(
                format!(
                "UPDATE finnel SET value = '{current}' WHERE key = 'version';"
            )
                .as_str(),
                (),
            )?;
        }

        Ok(())
    }

    fn upgrade_from(db: &Database, version: &Version) -> Result<()>;
}

impl Upgrade for Database {
    fn upgrade_from(db: &Database, version: &Version) -> Result<()> {
        crate::merchant::Merchant::upgrade_from(db, version)?;
        crate::category::Category::upgrade_from(db, version)?;
        //crate::account::Account::upgrade_from(db, version)?;
        //crate::account::Record::upgrade_from(db, version)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn open_memory() -> Result<()> {
        assert!(Database::memory().is_ok());
        assert_eq!(Database::memory()?.version()?, Version::new(0, 0, 0));

        Ok(())
    }

    #[test]
    fn setup() -> Result<()> {
        let db = Database::memory()?;

        assert_eq!(db.version()?, Version::new(0, 0, 0));

        Database::setup(&db)?;

        assert_eq!(db.version()?, Version::parse(env!("CARGO_PKG_VERSION"))?);

        Ok(())
    }
}
