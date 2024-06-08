use std::path::Path;

use semver::Version;

use rusqlite::Connection;

mod error;
pub use error::{Error, Result};

mod id;
pub use id::Id;

mod money;
pub use money::Money;

pub struct Database {
    pub connection: Connection,
}

impl From<Connection> for Database {
    fn from(connection: Connection) -> Self {
        Database { connection }
    }
}

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

        assert_eq!(db.version()?, Version::new(0, 0, 0));

        Database::setup(&db)?;

        assert_eq!(db.version()?, Version::parse(env!("CARGO_PKG_VERSION"))?);

        Ok(())
    }
}
