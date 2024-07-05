use crate::{Category, Database, Record};
use db::{Connection, Entity, Error, Id, Result, Row, Upgrade};

use derive::{Entity, EntityDescriptor};

mod query;
pub use query::QueryMerchant;

#[derive(Debug, Default, Entity, EntityDescriptor)]
#[entity(table = "merchants")]
pub struct Merchant {
    id: Option<Id>,
    pub name: String,
    default_category_id: Option<Id>,
}

impl Merchant {
    pub fn new<T: Into<String>>(name: T) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    pub fn default_category_id(&self) -> Option<Id> {
        self.default_category_id
    }

    /// Change the default category
    ///
    /// Passing a non-persisted category (i.e. without id) will instead reset
    /// the default_category_id to `None`
    pub fn set_default_category(&mut self, category: Option<&Category>) {
        self.default_category_id = category.and_then(Entity::id);
    }
}

impl Merchant {
    pub fn delete(&mut self, db: &mut Connection) -> Result<()> {
        if let Some(id) = self.id() {
            let tx = db.transaction()?;

            Record::clear_merchant_id(&tx, id)?;
            tx.execute(
                "DELETE FROM merchants
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
        let query = "SELECT * FROM merchants WHERE name = ? LIMIT 1;";
        let mut statement = db.prepare(query)?;

        match statement.query_row([name], |row| Self::try_from(&Row::from(row)))
        {
            Ok(record) => Ok(record),
            Err(rusqlite::Error::QueryReturnedNoRows) => Err(Error::NotFound),
            Err(e) => Err(e.into()),
        }
    }
}

impl Upgrade<Merchant> for Database {
    fn upgrade_from(&self, _version: &semver::Version) -> Result<()> {
        match self.execute(
            "CREATE TABLE IF NOT EXISTS merchants (
                id INTEGER NOT NULL PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                default_category_id INTEGER
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
    use crate::test::prelude::{assert_eq, assert_ne, Result, *};

    use crate::record::NewRecord;

    #[test]
    fn create_update() -> Result<()> {
        let db = test::db()?;

        let mut merchant = Merchant::new("Uraidla Pub");
        assert_eq!(None, merchant.id());
        merchant.save(&db)?;
        assert_eq!(Some(Id::from(1)), merchant.id());

        assert_eq!("Uraidla Pub", merchant.name);
        merchant.name = "Chariot".to_string();
        merchant.save(&db)?;
        assert_eq!("Chariot", Merchant::find(&db, Id::from(1))?.name);

        Ok(())
    }

    #[test]
    fn find_by_name() -> Result<()> {
        let db = test::db()?;
        let merchant = test::merchant(&db, "Foo")?;

        assert_eq!(merchant.id(), Merchant::find_by_name(&db, "Foo")?.id());

        Ok(())
    }

    #[test]
    fn delete() -> Result<()> {
        let mut db = test::db()?;
        let account = test::account(&db, "Cash")?;
        let mut merchant = test::merchant(&db, "Uraidla Pub")?;

        let mut record = NewRecord {
            account_id: account.id(),
            currency: account.currency,
            merchant_id: merchant.id(),
            ..Default::default()
        };
        let mut record = record.save(&db)?;

        merchant.delete(&mut db)?;
        assert_eq!(None, record.reload(&db)?.merchant_id());

        Ok(())
    }

    #[test]
    fn default_category() -> Result<()> {
        let db = test::db()?;

        let mut category = Category::new("foo");
        let mut merchant = Merchant::new("bar");

        merchant.set_default_category(Some(&category));
        assert_eq!(None, merchant.default_category_id());

        category.save(&db)?;
        merchant.set_default_category(Some(&category));
        assert_ne!(None, category.id());
        assert_eq!(category.id(), merchant.default_category_id());

        // Check that default_category_id is correctly persisted
        merchant.save(&db)?;
        assert_eq!(category.id(), merchant.reload(&db)?.default_category_id());

        Ok(())
    }
}
