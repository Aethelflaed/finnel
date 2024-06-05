use std::path::Path;

use chrono::{offset::Utc, DateTime};
use semver::Version;

use sqlite::{BindableWithIndex, Connection, ParameterIndex, State, Statement};

use crate::transaction;

#[derive(Copy, Clone, Debug, PartialEq, Eq, derive_more::From)]
pub struct Id(i64);

impl BindableWithIndex for Id {
    fn bind<T: ParameterIndex>(
        self,
        statement: &mut Statement<'_>,
        index: T,
    ) -> sqlite::Result<()> {
        self.0.bind(statement, index)
    }
}

#[derive(Copy, Clone, Debug, derive_more::From, derive_more::Into)]
pub struct Amount(oxydized_money::Amount);

impl Amount {
    pub fn try_read(field: &str, statement: &Statement) -> Result<Self> {
        Ok(Amount(oxydized_money::Amount(
            oxydized_money::Decimal::from_str_exact(
                &statement
                    .read::<String, _>(format!("{field}_val").as_str())?,
            )
            .unwrap(),
            oxydized_money::Currency::from_code(
                &statement
                    .read::<String, _>(format!("{field}_cur").as_str())?,
            )
            .unwrap(),
        )))
    }
}

#[derive(Copy, Clone, Debug, derive_more::From, derive_more::Into)]
pub struct Date(DateTime<Utc>);

impl Date {
    pub fn try_read(field: &str, statement: &Statement) -> Result<Self> {
        Ok(Date(
            statement
                .read::<String, _>(field)?
                .parse::<DateTime<Utc>>()?,
        ))
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
    Sqlite(#[from] sqlite::Error),
    #[error("Not found")]
    NotFound,
    #[error("Parsing date error")]
    DateParseError(#[from] chrono::ParseError),
    #[error("Parsing transaction type error")]
    TransactionTypeParseError(#[from] transaction::ParseTypeError),
    #[error("Parsing version information")]
    VersionError(#[from] semver::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Database {
    pub fn open<T: AsRef<Path>>(path: T) -> Result<Database> {
        sqlite::open(path).map(|c| c.into()).map_err(|e| e.into())
    }

    pub fn memory() -> Result<Database> {
        Self::open(":memory:")
    }

    // &mut self ensures the database cannot be borrowed twice
    pub fn transaction<T: FnOnce(&Database) -> Result<U>, U>(
        &mut self,
        block: T,
    ) -> Result<U> {
        self.connection.execute("BEGIN TRANSACTION")?;
        match block(self) {
            Ok(value) => match self.connection.execute("COMMIT") {
                Ok(_) => Ok(value),
                Err(e) => {
                    self.connection.execute("ROLLBACK")?;
                    Err(e.into())
                }
            },
            Err(e) => {
                self.connection.execute("ROLLBACK")?;
                Err(e)
            }
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

        if matches!(statement.next(), Ok(State::Done)) {
            return Ok(Version::new(0, 0, 0));
        }

        statement = self
            .connection
            .prepare("SELECT value FROM finnel WHERE key = 'version'")?;

        if let Ok(State::Row) = statement.next() {
            Ok(Version::parse(&statement.read::<String, _>("value")?)?)
        } else {
            Ok(Version::new(0, 0, 0))
        }
    }
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
            )?;
        }

        if version < current {
            Self::upgrade_from(db, &version)?;
        }

        if version == Version::new(0, 0, 0) {
            db.connection.execute(
                format!(
                    "INSERT INTO finnel (key, value) VALUES('version', '{current}');"
                )
            )?;
        } else {
            db.connection.execute(
                format!(
                    "UPDATE finnel SET value = '{current}' WHERE key = 'version';"
                )
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
        crate::account::Account::upgrade_from(db, version)?;
        crate::account::Record::upgrade_from(db, version)?;

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
        Database::setup(&db)
    }

    #[test]
    fn transaction_ok() {
        let mut db = Database::open(":memory:").unwrap();

        let result = db.transaction(|db| {
            db.connection.execute("CREATE TABLE test_table ( name );")?;
            db.connection
                .execute("INSERT INTO test_table(name) VALUES('bar')")?;
            Ok(true)
        });
        assert!(result.is_ok());

        let query = "SELECT * FROM test_table LIMIT 1";
        let mut statement = db.connection.prepare(query).unwrap();
        assert!(statement.next().is_ok());
    }

    #[test]
    fn transaction_err() {
        let mut db = Database::open(":memory:").unwrap();

        let result = db.transaction(|db| {
            db.connection.execute("CREATE TABLE test_table ( name );")?;
            db.connection
                .execute("INSERT INTO test_table(name) VALUES('bar')")?;
            Err::<bool, _>(Error::NotFound)
        });
        assert!(result.is_err());

        let query = "SELECT * FROM test_table LIMIT 1";
        assert!(db.connection.prepare(query).is_err());
    }
}
