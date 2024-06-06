use crate::database::{
    Amount as DbAmount, Database, Entity, Error, Readable, Result, Upgrade,
};
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
        Account {
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
        statement.bind((1, name))?;

        if let Ok(sqlite::State::Row) = statement.next() {
            statement.try_into()
        } else {
            Err(Error::NotFound)
        }
    }

    pub fn find_or_create_by_name<T: Into<String>>(
        db: &Database,
        name: T,
    ) -> Result<Account> {
        let name_string: String = name.into();

        match Self::find_by_name(db, name_string.as_str()) {
            Err(Error::NotFound) => {
                let mut account = Self::new(name_string);
                account.save(&db)?;
                Ok(account)
            }
            value => value,
        }
    }
}

impl Default for Account {
    fn default() -> Self {
        Account {
            id: None,
            name: String::from(""),
            balance: DbAmount::default().into(),
        }
    }
}

impl Entity for Account {
    fn id(&self) -> Option<Id> {
        self.id
    }

    fn find(db: &Database, id: Id) -> Result<Self> {
        let query = "SELECT * FROM accounts WHERE id = ? LIMIT 1;";
        let mut statement = db.connection.prepare(query)?;
        statement.bind((1, id))?;

        if let Ok(sqlite::State::Row) = statement.next() {
            statement.try_into()
        } else {
            Err(Error::NotFound)
        }
    }

    fn save(&mut self, db: &Database) -> Result<()> {
        if let Some(id) = self.id {
            let query = "UPDATE accounts SET
                    name = :name,
                    balance_val = :balance_val,
                    balance_cur = :balance_cur
                WHERE id = :id";
            let mut statement = db.connection.prepare(query)?;
            statement.bind((":name", self.name.as_str()))?;
            let db_amount = DbAmount::from(self.balance);
            statement.bind((":balance_val", db_amount.val().as_str()))?;
            statement.bind((":balance_cur", db_amount.cur()))?;
            statement.bind((":id", id))?;

            if let Ok(sqlite::State::Done) = statement.next() {
                Ok(())
            } else {
                Err(Error::NotFound)
            }
        } else {
            let query = "INSERT INTO accounts (name, balance_val, balance_cur)
                VALUES(?, ?, ?) RETURNING id;";
            let mut statement = db.connection.prepare(query)?;
            statement.bind((1, self.name.as_str()))?;

            let db_amount = DbAmount::from(self.balance);
            statement.bind((2, db_amount.val().as_str()))?;
            statement.bind((3, db_amount.cur()))?;

            if let Ok(sqlite::State::Row) = statement.next() {
                self.id = Some(Id::try_read("id", &statement)?);
                Ok(())
            } else {
                Err(Error::NotFound)
            }
        }
    }
}

impl TryFrom<sqlite::Statement<'_>> for Account {
    type Error = Error;

    fn try_from(statement: sqlite::Statement) -> Result<Self> {
        Ok(Account {
            id: Some(Id::try_read("id", &statement)?),
            name: statement.read::<String, _>("name")?,
            balance: DbAmount::try_read("balance", &statement)?.into(),
        })
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
