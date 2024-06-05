use crate::database::{Amount as DbAmount, Database, Error, Result, Upgrade};
use oxydized_money::Amount;

pub use crate::database::Id;

mod record;
pub use record::{Record, RecordStorage};

#[derive(Debug)]
pub struct Account {
    id: Id,
    name: String,
    balance: Amount,
}

impl Account {
    pub fn get_id(&self) -> Id {
        self.id
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_balance(&self) -> Amount {
        self.balance
    }
}

impl TryFrom<sqlite::Statement<'_>> for Account {
    type Error = Error;

    fn try_from(statement: sqlite::Statement) -> Result<Self> {
        Ok(Account {
            id: Id::from(statement.read::<i64, _>("id")?),
            name: statement.read::<String, _>("name")?,
            balance: DbAmount::try_read("balance", &statement)?.into(),
        })
    }
}

pub trait AccountStorage {
    fn find(&self, id: Id) -> Result<Account>;
    fn find_by_name(&self, name: &str) -> Result<Account>;
    fn find_or_create_by_name(&self, name: &str) -> Result<Account>;
    fn create(&self, name: &str) -> Result<Account>;
}

impl AccountStorage for Database {
    fn find(&self, id: Id) -> Result<Account> {
        let query = "SELECT * FROM accounts WHERE id = ? LIMIT 1;";
        let mut statement = self.connection.prepare(query).unwrap();
        statement.bind((1, id)).unwrap();

        if let Ok(sqlite::State::Row) = statement.next() {
            statement.try_into()
        } else {
            Err(Error::NotFound)
        }
    }

    fn find_by_name(&self, name: &str) -> Result<Account> {
        let query = "SELECT * FROM accounts WHERE name = ? LIMIT 1;";
        let mut statement = self.connection.prepare(query).unwrap();
        statement.bind((1, name)).unwrap();

        if let Ok(sqlite::State::Row) = statement.next() {
            statement.try_into()
        } else {
            Err(Error::NotFound)
        }
    }

    fn find_or_create_by_name(&self, name: &str) -> Result<Account> {
        match self.find_by_name(name) {
            Err(Error::NotFound) => self.create(name),
            value => value,
        }
    }

    fn create(&self, name: &str) -> Result<Account> {
        let query = "INSERT INTO accounts(name) VALUES(?) RETURNING *;";
        let mut statement = self.connection.prepare(query).unwrap();
        statement.bind((1, name)).unwrap();

        if let Ok(sqlite::State::Row) = statement.next() {
            statement.try_into()
        } else {
            Err(Error::NotFound)
        }
    }
}

impl Upgrade for Account {
    fn upgrade_from(db: &Database, _version: &semver::Version) -> Result<()> {
        db.connection
            .execute(
                "
                CREATE TABLE IF NOT EXISTS accounts (
                    id INTEGER NOT NULL PRIMARY KEY,
                    name TEXT NOT NULL UNIQUE,
                    balance_val TEXT NOT NULL DEFAULT '0',
                    balance_cur TEXT NOT NULL DEFAULT 'EUR'
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
        Account::setup(&db).unwrap();

        let account = db.create("Uraidla Pub").unwrap();
        assert_eq!(Id::from(1), account.get_id());
        assert_eq!("Uraidla Pub", account.get_name());

        assert_eq!(
            "Uraidla Pub",
            AccountStorage::find(&db, Id::from(1)).unwrap().get_name()
        );

        assert_eq!(
            Id::from(1),
            db.find_by_name("Uraidla Pub").unwrap().get_id()
        );
    }

    #[test]
    fn find_or_create_by_name() {
        let db = Database::memory().unwrap();
        Account::setup(&db).unwrap();

        let res = db.find_by_name("Chariot");
        assert!(matches!(res.unwrap_err(), Error::NotFound));

        let account = db.find_or_create_by_name("Chariot").unwrap();
        assert_eq!(Id::from(1), account.get_id());
        assert_eq!("Chariot", account.get_name());

        assert!(db.create("Chariot").is_err());
        assert!(db.find_by_name("Chariot").is_ok());
    }
}
