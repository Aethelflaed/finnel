use crate::database::{Database, Error, Result, Upgrade};

pub use crate::database::Id;

#[derive(Debug)]
pub struct Merchant {
    id: Id,
    name: String,
}

impl Merchant {
    pub fn get_id(&self) -> Id {
        self.id
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
}

impl TryFrom<sqlite::Statement<'_>> for Merchant {
    type Error = Error;

    fn try_from(statement: sqlite::Statement) -> Result<Self> {
        Ok(Merchant {
            id: Id::from(statement.read::<i64, _>("id")?),
            name: statement.read::<String, _>("name")?,
        })
    }
}

pub trait MerchantStorage {
    fn find(&self, id: Id) -> Result<Merchant>;
    fn find_by_name(&self, name: &str) -> Result<Merchant>;
    fn find_or_create_by_name(&self, name: &str) -> Result<Merchant>;
    fn create(&self, name: &str) -> Result<Merchant>;
}

impl MerchantStorage for Database {
    fn find(&self, id: Id) -> Result<Merchant> {
        let query = "SELECT * FROM merchants WHERE id = ? LIMIT 1;";
        let mut statement = self.connection.prepare(query).unwrap();
        statement.bind((1, id)).unwrap();

        if let Ok(sqlite::State::Row) = statement.next() {
            statement.try_into()
        } else {
            Err(Error::NotFound)
        }
    }

    fn find_by_name(&self, name: &str) -> Result<Merchant> {
        let query = "SELECT * FROM merchants WHERE name = ? LIMIT 1;";
        let mut statement = self.connection.prepare(query).unwrap();
        statement.bind((1, name)).unwrap();

        if let Ok(sqlite::State::Row) = statement.next() {
            statement.try_into()
        } else {
            Err(Error::NotFound)
        }
    }

    fn find_or_create_by_name(&self, name: &str) -> Result<Merchant> {
        match self.find_by_name(name) {
            Err(Error::NotFound) => self.create(name),
            value => value,
        }
    }

    fn create(&self, name: &str) -> Result<Merchant> {
        let query = "INSERT INTO merchants(name) VALUES(?) RETURNING *;";
        let mut statement = self.connection.prepare(query).unwrap();
        statement.bind((1, name)).unwrap();

        if let Ok(sqlite::State::Row) = statement.next() {
            statement.try_into()
        } else {
            Err(Error::NotFound)
        }
    }
}

impl Upgrade for Merchant {
    fn upgrade_from(db: &Database, _version: &semver::Version) -> Result<()> {
        db.connection
            .execute(
                "
                CREATE TABLE IF NOT EXISTS merchants (
                    id INTEGER NOT NULL PRIMARY KEY,
                    name TEXT NOT NULL UNIQUE
                );
            ",
            )
            .map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn create() {
        let db = Database::memory().unwrap();
        Merchant::setup(&db).unwrap();

        let merchant = db.create("Uraidla Pub").unwrap();
        assert_eq!(Id::from(1), merchant.get_id());
        assert_eq!("Uraidla Pub", merchant.get_name());

        assert_eq!("Uraidla Pub", db.find(Id::from(1)).unwrap().get_name());

        assert_eq!(
            Id::from(1),
            db.find_by_name("Uraidla Pub").unwrap().get_id()
        );
    }

    #[test]
    fn find_or_create_by_name() {
        let db = Database::memory().unwrap();
        Merchant::setup(&db).unwrap();

        let res = db.find_by_name("Chariot");
        assert!(matches!(res.unwrap_err(), Error::NotFound));

        let merchant = db.find_or_create_by_name("Chariot").unwrap();
        assert_eq!(Id::from(1), merchant.get_id());
        assert_eq!("Chariot", merchant.get_name());

        assert!(db.create("Chariot").is_err());
        assert!(db.find_by_name("Chariot").is_ok());
    }
}
