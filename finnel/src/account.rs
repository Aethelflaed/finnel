use crate::Database;
use db::{
    self as database, Connection, Entity, Error, Id, Result, Row, Upgrade,
};
use oxydized_money::{Amount, Currency, Decimal};

use crate::record::Record;

use derive::{Entity, EntityDescriptor};

#[derive(Debug, Entity, EntityDescriptor)]
#[entity(table = "accounts")]
pub struct Account {
    id: Option<Id>,
    pub name: String,
    #[field(db_type = database::Decimal)]
    balance: Decimal,
    #[field(db_type = database::Currency, update = false)]
    pub(crate) currency: Currency,
}

impl Account {
    pub fn new<T: Into<String>>(name: T) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
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

        match statement.query_row([name], |row| Self::try_from(&Row::from(row)))
        {
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
            .query_and_then([], |row| Self::try_from(&Row::from(row)))
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

impl Upgrade<Account> for Database {
    fn upgrade_from(&self, _version: &semver::Version) -> Result<()> {
        match self.execute(
            "CREATE TABLE IF NOT EXISTS accounts (
                id INTEGER NOT NULL PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                balance INTEGER NOT NULL,
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
    use crate::test::prelude::{assert_eq, Result, *};

    #[test]
    fn crud() -> Result<()> {
        let db = &mut test::db()?;

        let mut account = Account::new("Uraidla Pub");
        assert_eq!(None, account.id());
        account.save(&db)?;
        assert_eq!(Some(Id::from(1)), account.id());

        assert_eq!("Uraidla Pub", account.name);
        account.name = "Chariot".to_string();
        account.save(&db)?;
        assert_eq!("Chariot", account.reload(&db)?.name);

        assert_eq!(Decimal::ZERO, account.balance);

        let mut record = test::record(db, &account)?;
        account.delete(db)?;

        assert!(record.reload(db).is_err());
        assert!(account.reload(db).is_err());

        Ok(())
    }

    #[test]
    fn for_each() -> Result<()> {
        let db = Database::memory()?;
        db.setup()?;

        let mut account1 = Account::new("Account 1");
        account1.save(&db)?;
        let mut account2 = Account::new("Account 2");
        account2.save(&db)?;

        let mut accounts = Vec::new();
        Account::for_each(&db, |account| {
            accounts.push(account);
        })?;

        assert_eq!("Account 1", accounts[0].name);
        assert_eq!("Account 2", accounts[1].name);

        Ok(())
    }
}
