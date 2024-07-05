use chrono::{offset::Utc, DateTime};

use oxydized_money::{Amount, Currency, Decimal};

use crate::Database;
use db::{
    self as database, Connection, Entity, Error, Id, Result, Row, Upgrade,
};

use crate::category::Category;
use crate::merchant::Merchant;
use crate::transaction::{Direction, Mode};

mod new;
mod query;

pub use new::NewRecord;
pub use query::{FullRecord, QueryRecord};

use derive::{Entity, EntityDescriptor};

#[derive(Debug, Entity, EntityDescriptor)]
#[entity(table = "records")]
pub struct Record {
    id: Option<Id>,
    #[field(update = false)]
    account_id: Id,
    #[field(db_type = database::Decimal, update = false)]
    amount: Decimal,
    #[field(db_type = database::Currency, update = false)]
    currency: Currency,
    #[field(update = false)]
    operation_date: DateTime<Utc>,
    pub value_date: DateTime<Utc>,
    #[field(update = false)]
    direction: Direction,
    #[field(update = false)]
    mode: Mode,
    pub details: String,
    category_id: Option<Id>,
    merchant_id: Option<Id>,
}

impl Record {
    pub fn amount(&self) -> Amount {
        Amount(self.amount, self.currency)
    }

    pub fn operation_date(&self) -> DateTime<Utc> {
        self.operation_date
    }

    pub fn direction(&self) -> Direction {
        self.direction
    }

    pub fn mode(&self) -> Mode {
        self.mode.clone()
    }

    pub fn details(&self) -> &str {
        self.details.as_str()
    }

    pub fn category_id(&self) -> Option<Id> {
        self.category_id
    }

    pub fn set_category(&mut self, category: Option<&Category>) {
        self.category_id = category.and_then(Entity::id);
    }

    pub fn merchant_id(&self) -> Option<Id> {
        self.merchant_id
    }

    pub fn set_merchant(&mut self, merchant: Option<&Merchant>) {
        self.merchant_id = merchant.and_then(Entity::id);
    }
}

impl Record {
    pub(crate) fn clear_merchant_id(
        db: &Connection,
        merchant_id: Id,
    ) -> Result<()> {
        db.execute(
            "UPDATE records
            SET merchant_id = NULL
            WHERE merchant_id = :merchant_id",
            rusqlite::named_params! {":merchant_id": merchant_id},
        )?;
        Ok(())
    }

    pub(crate) fn clear_category_id(
        db: &Connection,
        category_id: Id,
    ) -> Result<()> {
        db.execute(
            "UPDATE records
            SET category_id = NULL
            WHERE category_id = :category_id",
            rusqlite::named_params! {":category_id": category_id},
        )?;
        Ok(())
    }

    pub(crate) fn delete_by_account_id(
        db: &Connection,
        account_id: Id,
    ) -> Result<()> {
        db.execute(
            "DELETE FROM records
            WHERE account_id = :account_id",
            rusqlite::named_params! {":account_id": account_id},
        )?;
        Ok(())
    }
}

impl Upgrade<Record> for Database {
    fn upgrade_from(&self, _version: &semver::Version) -> Result<()> {
        match self.execute(
            "CREATE TABLE IF NOT EXISTS records (
                    id INTEGER NOT NULL PRIMARY KEY,
                    account_id INTEGER NOT NULL,
                    amount INTEGER NOT NULL,
                    currency TEXT NOT NULL,
                    operation_date TEXT NOT NULL,
                    value_date TEXT NOT NULL,
                    direction TEXT NOT NULL DEFAULT 'Debit',
                    mode TEXT NOT NULL DEFAULT 'Direct',
                    details TEXT NOT NULL DEFAULT '',
                    category_id INTEGER,
                    merchant_id INTEGER
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
    fn update() -> Result<()> {
        let db = &test::db()?;
        let account = test::account(db, "Cash")?;
        let mut record = test::record(db, &account)?;

        let category = test::category(db, "Foo")?;
        record.set_category(Some(&category));
        record.save(&db)?;

        let merchant = test::merchant(db, "Bar")?;
        record.set_merchant(Some(&merchant));
        record.save(&db)?;

        record.reload(&db)?;
        assert_eq!(category.id(), record.category_id());
        assert_eq!(merchant.id(), record.merchant_id());

        Ok(())
    }

    #[test]
    fn clear_merchant_id() -> Result<()> {
        let db = &mut test::db()?;
        let account = test::account(db, "Cash")?;
        let merchant_1 = test::merchant(db, "Foo")?;
        let merchant_2 = test::merchant(db, "Bar")?;

        let mut record_1 = NewRecord::new(&account);
        record_1.merchant_id = Some(merchant_1.id().unwrap());
        let mut record_1 = record_1.save(db)?;

        let mut record_2 = NewRecord::new(&account);
        record_2.merchant_id = Some(merchant_2.id().unwrap());
        let mut record_2 = record_2.save(db)?;

        Record::clear_merchant_id(db, merchant_1.id().unwrap())?;
        assert_eq!(None, record_1.reload(db)?.merchant_id());
        assert_eq!(merchant_2.id(), record_2.reload(db)?.merchant_id());

        Ok(())
    }

    #[test]
    fn clear_category_id() -> Result<()> {
        let db = &mut test::db()?;
        let account = test::account(db, "Cash")?;
        let category_1 = test::category(db, "Foo")?;
        let category_2 = test::category(db, "Bar")?;

        let mut record_1 = NewRecord::new(&account);
        record_1.category_id = Some(category_1.id().unwrap());
        let mut record_1 = record_1.save(db)?;

        let mut record_2 = NewRecord::new(&account);
        record_2.category_id = Some(category_2.id().unwrap());
        let mut record_2 = record_2.save(db)?;

        Record::clear_category_id(db, category_1.id().unwrap())?;
        assert_eq!(None, record_1.reload(db)?.category_id());
        assert_eq!(category_2.id(), record_2.reload(db)?.category_id());

        Ok(())
    }

    #[test]
    fn delete_by_account_id() -> Result<()> {
        let db = &mut test::db()?;
        let account_1 = test::account(db, "Cash")?;
        let account_2 = test::account(db, "Account")?;

        let mut record_1 = test::record(db, &account_1)?;
        let mut record_2 = test::record(db, &account_2)?;

        Record::delete_by_account_id(db, account_1.id().unwrap())?;
        assert!(record_1.reload(db).is_err());
        assert!(record_2.reload(db).is_ok());

        Ok(())
    }
}
