use crate::database::{Database, Entity, Error, Money, Result, Upgrade};
use oxydized_money::Amount;

pub use crate::database::Id;

mod record;
pub use record::Record;

#[derive(Debug)]
pub struct Account {
    id: Option<Id>,
    name: String,
    balance: Amount,
}

impl Account {
    pub fn new<T: Into<String>>(name: T) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name<T: Into<String>>(&mut self, name: T) {
        self.name = name.into();
    }

    pub fn balance(&self) -> Amount {
        self.balance
    }

    pub fn find_by_name(db: &Database, name: &str) -> Result<Self> {
        let query = "SELECT * FROM accounts WHERE name = ? LIMIT 1;";
        let mut statement = db.connection.prepare(query)?;

        match statement.query_row([name], |row| row.try_into()) {
            Ok(record) => Ok(record),
            Err(rusqlite::Error::QueryReturnedNoRows) => Err(Error::NotFound),
            Err(e) => Err(e.into()),
        }
    }

    pub fn find_or_create_by_name<T: Into<String>>(
        db: &Database,
        name: T,
    ) -> Result<Self> {
        let name_string: String = name.into();

        match Self::find_by_name(db, name_string.as_str()) {
            Err(Error::NotFound) => {
                let mut record = Self::new(name_string);
                record.save(db)?;
                Ok(record)
            }
            value => value,
        }
    }
}

impl Default for Account {
    fn default() -> Self {
        Self {
            id: None,
            name: String::new(),
            balance: Money::default().into(),
        }
    }
}

impl TryFrom<&rusqlite::Row<'_>> for Account {
    type Error = rusqlite::Error;

    fn try_from(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Account {
            id: row.get("id")?,
            name: row.get("name")?,
            balance: row.get::<&str, Money>("balance")?.into(),
        })
    }
}

impl Entity for Account {
    fn id(&self) -> Option<Id> {
        self.id
    }

    fn find(db: &Database, id: Id) -> Result<Self> {
        let query = "SELECT * FROM accounts WHERE id = ? LIMIT 1;";
        let mut statement = db.connection.prepare(query)?;
        match statement.query_row([id], |row| row.try_into()) {
            Ok(record) => Ok(record),
            Err(rusqlite::Error::QueryReturnedNoRows) => Err(Error::NotFound),
            Err(e) => Err(e.into()),
        }
    }

    fn save(&mut self, db: &Database) -> Result<()> {
        use rusqlite::named_params;

        if let Some(id) = self.id() {
            let query = "
                UPDATE accounts
                SET
                    name = :name,
                    balance = :balance
                WHERE
                    id = :id";
            let mut statement = db.connection.prepare(query)?;
            let params = named_params! {
                ":id": id,
                ":name": self.name,
                ":balance": Money::from(self.balance)
            };
            match statement.execute(params) {
                Ok(_) => Ok(()),
                Err(e) => Err(e.into()),
            }
        } else {
            let query = "
                INSERT INTO accounts (
                    name,
                    balance
                )
                VALUES (
                    :name, :balance
                )
                RETURNING id;";
            let mut statement = db.connection.prepare(query)?;
            let params = named_params! {
                ":name": self.name.as_str(),
                ":balance": Money::from(self.balance)
            };

            Ok(statement.query_row(params, |row| {
                self.id = row.get(0)?;
                Ok(())
            })?)
        }
    }
}

impl Upgrade for Account {
    fn upgrade_from(db: &Database, _version: &semver::Version) -> Result<()> {
        match db.connection.execute(
            "CREATE TABLE IF NOT EXISTS accounts (
                id INTEGER NOT NULL PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                balance BLOB NOT NULL
            );",
            (),
        ) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn crud() -> Result<()> {
        let db = Database::memory()?;
        Account::setup(&db)?;

        let mut account = Account::new("Uraidla Pub");
        assert_eq!(None, account.id());
        account.save(&db)?;
        assert_eq!(Some(Id::from(1)), account.id());

        assert_eq!("Uraidla Pub", account.name());
        account.set_name("Chariot");
        account.save(&db)?;
        assert_eq!("Chariot", Account::find(&db, Id::from(1))?.name());

        Ok(())
    }

    #[test]
    fn find_or_create_by_name() -> Result<()> {
        let db = Database::memory()?;
        Account::setup(&db)?;

        assert!(matches!(
            Account::find_by_name(&db, "Chariot"),
            Err(Error::NotFound)
        ));

        let mut account = Account::new("Chariot");
        account.save(&db)?;

        assert_eq!(account.id(), Account::find_by_name(&db, "Chariot")?.id());

        account = Account::find_or_create_by_name(&db, "Uraidla Pub")?;
        assert_eq!(Some(Id::from(2)), account.id());

        Ok(())
    }
}
