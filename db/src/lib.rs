use std::path::Path;

use semver::Version;

pub use rusqlite::Connection;

mod error;
pub use error::{Error, Result};

mod id;
pub use id::Id;

mod money;
pub use money::{Currency, Decimal};

mod query;
pub use query::Query;

pub trait DatabaseTrait:
    From<Connection>
    + Into<Connection>
    + core::ops::Deref<Target = Connection>
    + core::ops::DerefMut<Target = Connection>
{
    fn open<T: AsRef<Path>>(path: T) -> Result<Self> {
        match Connection::open(path) {
            Ok(connection) => Ok(connection.into()),
            Err(e) => Err(e.into()),
        }
    }

    fn memory() -> Result<Self> {
        match Connection::open_in_memory() {
            Ok(connection) => Ok(connection.into()),
            Err(e) => Err(e.into()),
        }
    }

    fn get<K>(&self, key: K) -> Result<Option<String>>
    where
        K: AsRef<str> + rusqlite::ToSql,
    {
        match self
            .prepare("SELECT value FROM finnel WHERE key = ?")?
            .query_row([key], |row| row.get::<usize, String>(0))
        {
            Ok(record) => Ok(Some(record)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn set<K, V>(&self, key: K, value: V) -> Result<()>
    where
        K: AsRef<str> + rusqlite::ToSql,
        V: AsRef<str> + rusqlite::ToSql,
    {
        self.execute(
            "INSERT INTO finnel(key, value)
                VALUES(:key, :value)
                ON CONFLICT(key)
                DO UPDATE SET value = :value",
            rusqlite::named_params! {":key": key, ":value": value},
        )?;
        Ok(())
    }

    fn reset<K>(&self, key: K) -> Result<()>
    where
        K: AsRef<str> + rusqlite::ToSql,
    {
        self.execute(
            "DELETE FROM finnel
            WHERE key = :key",
            rusqlite::named_params! {":key": key},
        )?;
        Ok(())
    }

    fn version(&self) -> Result<Version> {
        let mut statement = self.prepare(
            "
        SELECT 
            name
        FROM
            sqlite_schema
        WHERE 
            name = 'finnel';",
        )?;

        {
            let mut rows = statement.query([])?;

            if rows.next()?.is_none() {
                return Ok(Version::new(0, 0, 0));
            }
        }

        if let Some(version) = self.get("version")? {
            Ok(Version::parse(&version)?)
        } else {
            Ok(Version::new(0, 0, 0))
        }
    }

    fn current_version(&self) -> Result<Version> {
        Ok(Version::parse(env!("CARGO_PKG_VERSION"))?)
    }

    fn setup(&self) -> Result<()> {
        let version = self.version()?;
        let current = self.current_version()?;

        if version == Version::new(0, 0, 0) {
            self.execute(
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
            self.upgrade_from(&version)?;
        }

        self.set("version", current.to_string())
    }

    fn upgrade_from(&self, version: &Version) -> Result<()>;
}

pub trait Entity: Sized {
    fn id(&self) -> Option<Id>;

    fn find(db: &Connection, id: Id) -> Result<Self>;
    fn save(&mut self, db: &Connection) -> Result<()>;
}

pub trait Upgrade<T> {
    fn upgrade_from(&self, version: &Version) -> Result<()>;
}

#[derive(
    derive_more::From,
    derive_more::Into,
    derive_more::Deref,
    derive_more::DerefMut,
)]
struct SimpleDatabase(Connection);

impl DatabaseTrait for SimpleDatabase {
    fn upgrade_from(&self, _version: &Version) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn open_memory() -> Result<()> {
        assert!(SimpleDatabase::memory().is_ok());
        assert_eq!(SimpleDatabase::memory()?.version()?, Version::new(0, 0, 0));

        Ok(())
    }

    #[test]
    fn setup() -> Result<()> {
        let db: SimpleDatabase = SimpleDatabase::memory()?.into();

        assert_eq!(db.version()?, Version::new(0, 0, 0));

        db.setup()?;

        assert_eq!(db.version()?, Version::parse(env!("CARGO_PKG_VERSION"))?);

        Ok(())
    }

    #[test]
    fn get_set_reset() -> Result<()> {
        let db: SimpleDatabase = SimpleDatabase::memory()?.into();
        db.setup()?;

        assert_eq!(None, db.get("foo")?);
        db.set("foo", "bar")?;
        assert_eq!(Some("bar"), db.get("foo")?.as_deref());
        db.reset("foo")?;
        assert_eq!(None, db.get("foo")?);

        Ok(())
    }
}
