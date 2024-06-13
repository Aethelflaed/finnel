use crate::database::{
    self, Connection, Database, Entity, Error, Id, Result, Upgrade,
};
use oxydized_money::{Amount, Currency, Decimal};

mod record;
pub use record::Record;

#[derive(Debug)]
pub struct Account {
    id: Option<Id>,
    name: String,
    balance: Decimal,
    currency: Currency,
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
        Amount(self.balance, self.currency)
    }

    pub fn currency(&self) -> Currency {
        self.currency
    }

    pub fn delete(&mut self, db: &mut Connection) -> Result<()> {
        if let Some(id) = self.id() {
            let tx = db.transaction()?;
            Record::delete_by_account_id(&tx, id)?;
            tx.execute(
                "DELETE FROM accounts
                WHERE id = :id",
                rusqlite::named_params! {":id": id},
            )?;

            tx.commit()?;
            Ok(())
        } else {
            Err(Error::NotPersisted)
        }
    }

    pub fn find_by_name(db: &Connection, name: &str) -> Result<Self> {
        let query = "SELECT * FROM accounts WHERE name = ? LIMIT 1;";
        let mut statement = db.prepare(query)?;

        match statement.query_row([name], |row| row.try_into()) {
            Ok(record) => Ok(record),
            Err(rusqlite::Error::QueryReturnedNoRows) => Err(Error::NotFound),
            Err(e) => Err(e.into()),
        }
    }

    pub fn for_each<F>(db: &Connection, mut f: F) -> Result<()>
    where
        F: FnMut(Self),
    {
        match db
            .prepare("SELECT * FROM accounts")?
            .query_and_then([], |row| Self::try_from(row))
        {
            Ok(iter) => {
                for entity in iter {
                    f(entity?);
                }
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }
}

impl Default for Account {
    fn default() -> Self {
        Self {
            id: None,
            name: String::new(),
            balance: Decimal::ZERO,
            currency: Currency::EUR,
        }
    }
}

impl TryFrom<&rusqlite::Row<'_>> for Account {
    type Error = rusqlite::Error;

    fn try_from(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Account {
            id: row.get("id")?,
            name: row.get("name")?,
            balance: row.get::<&str, database::Decimal>("balance")?.into(),
            currency: row.get::<&str, database::Currency>("currency")?.into(),
        })
    }
}

impl Entity for Account {
    fn id(&self) -> Option<Id> {
        self.id
    }

    fn find(db: &Connection, id: Id) -> Result<Self> {
        let query = "SELECT * FROM accounts WHERE id = ? LIMIT 1;";
        let mut statement = db.prepare(query)?;
        match statement.query_row([id], |row| row.try_into()) {
            Ok(record) => Ok(record),
            Err(rusqlite::Error::QueryReturnedNoRows) => Err(Error::NotFound),
            Err(e) => Err(e.into()),
        }
    }

    fn save(&mut self, db: &Connection) -> Result<()> {
        use rusqlite::named_params;

        if let Some(id) = self.id() {
            let query = "
                UPDATE accounts
                SET
                    name = :name,
                    balance = :balance
                WHERE
                    id = :id";
            let mut statement = db.prepare(query)?;
            let params = named_params! {
                ":id": id,
                ":name": self.name,
                ":balance": database::Decimal::from(self.balance),
            };
            match statement.execute(params) {
                Ok(_) => Ok(()),
                Err(e) => Err(e.into()),
            }
        } else {
            let query = "
                INSERT INTO accounts (
                    name,
                    balance,
                    currency
                )
                VALUES (
                    :name, :balance, :currency
                )
                RETURNING id;";
            let mut statement = db.prepare(query)?;
            let params = named_params! {
                ":name": self.name.as_str(),
                ":balance": database::Decimal::from(self.balance),
                ":currency": database::Currency::from(self.currency),
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
        match db.execute(
            "CREATE TABLE IF NOT EXISTS accounts (
                id INTEGER NOT NULL PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                balance TEXT NOT NULL,
                currency TEXT NOT NULL
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

        assert_eq!(Decimal::ZERO, account.balance);

        Ok(())
    }

    #[test]
    fn for_each() -> Result<()> {
        let db = Database::memory()?;
        Account::setup(&db)?;

        let mut account1 = Account::new("Account 1");
        account1.save(&db)?;
        let mut account2 = Account::new("Account 2");
        account2.save(&db)?;

        let mut accounts = Vec::new();
        Account::for_each(&db, |account| {
            accounts.push(account);
        })?;

        assert_eq!("Account 1", accounts[0].name());
        assert_eq!("Account 2", accounts[1].name());

        Ok(())
    }
}
