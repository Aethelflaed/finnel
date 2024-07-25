use crate::{
    account::Account, category::Category, essentials::*, merchant::Merchant, schema::records,
    Amount, Currency, Decimal,
};

use chrono::{offset::Utc, DateTime};
use diesel::prelude::*;

mod new;
pub use new::NewRecord;

mod direction;
pub use direction::Direction;

mod mode;
pub use mode::{Mode, PaymentMethod};

pub mod query;
pub use query::QueryRecord;

mod consolidate;
pub use consolidate::consolidate;

#[derive(Debug, Queryable, Selectable, Identifiable, Associations)]
#[diesel(table_name = records)]
#[diesel(belongs_to(Account, foreign_key = account_id))]
#[diesel(belongs_to(Category, foreign_key = category_id))]
#[diesel(belongs_to(Merchant, foreign_key = merchant_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Record {
    pub id: i64,
    pub account_id: i64,
    #[diesel(deserialize_as = crate::db::Decimal)]
    pub amount: Decimal,
    #[diesel(deserialize_as = crate::db::Currency)]
    pub currency: Currency,
    pub operation_date: DateTime<Utc>,
    pub value_date: DateTime<Utc>,
    pub direction: Direction,
    pub mode: Mode,
    pub details: String,
    pub category_id: Option<i64>,
    pub merchant_id: Option<i64>,
}

impl Record {
    pub fn amount(&self) -> Amount {
        Amount(self.amount, self.currency)
    }

    pub fn find(conn: &mut Conn, id: i64) -> Result<Self> {
        records::table
            .find(id)
            .select(Record::as_select())
            .first(conn)
            .map_err(|e| e.into())
    }

    pub fn delete(&mut self, conn: &mut Conn) -> Result<()> {
        diesel::delete(&*self).execute(conn)?;

        Ok(())
    }
}

#[derive(Default, Clone, AsChangeset)]
#[diesel(table_name = records)]
pub struct ChangeRecord<'a> {
    #[diesel(serialize_as = crate::db::Decimal)]
    pub amount: Option<Decimal>,
    pub operation_date: Option<DateTime<Utc>>,
    pub value_date: Option<DateTime<Utc>>,
    pub direction: Option<Direction>,
    pub mode: Option<Mode>,
    pub details: Option<&'a str>,
    pub category_id: Option<Option<i64>>,
    pub merchant_id: Option<Option<i64>>,
}

impl ChangeRecord<'_> {
    pub fn save(self, conn: &mut Conn, record: &Record) -> Result<()> {
        diesel::update(record).set(self).execute(conn)?;
        Ok(())
    }

    pub fn apply(self, conn: &mut Conn, record: &mut Record) -> Result<()> {
        self.clone().save(conn, record)?;

        if let Some(value) = self.amount {
            record.amount = value;
        }
        if let Some(value) = self.operation_date {
            record.operation_date = value;
        }
        if let Some(value) = self.value_date {
            record.value_date = value;
        }
        if let Some(value) = self.direction {
            record.direction = value;
        }
        if let Some(value) = self.mode {
            record.mode = value;
        }
        if let Some(value) = self.details {
            record.details = value.to_string();
        }
        if let Some(value) = self.category_id {
            record.category_id = value;
        }
        if let Some(value) = self.merchant_id {
            record.merchant_id = value;
        }

        Ok(())
    }
}

pub(crate) fn clear_category_id(conn: &mut Conn, id: i64) -> Result<()> {
    diesel::update(records::table)
        .filter(records::category_id.eq(id))
        .set(records::category_id.eq(None::<i64>))
        .execute(conn)?;
    Ok(())
}

pub(crate) fn clear_merchant_id(conn: &mut Conn, id: i64) -> Result<()> {
    diesel::update(records::table)
        .filter(records::merchant_id.eq(id))
        .set(records::merchant_id.eq(None::<i64>))
        .execute(conn)?;
    Ok(())
}

pub(crate) fn delete_by_account_id(conn: &mut Conn, id: i64) -> Result<()> {
    diesel::delete(records::table)
        .filter(records::account_id.eq(id))
        .execute(conn)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::prelude::{assert_eq, Result, *};

    #[test]
    fn update() -> Result<()> {
        let db = &mut test::db()?;
        let account = test::account(db, "Cash")?;
        let mut record = test::record(db, &account)?;

        let category = test::category(db, "Foo")?;
        diesel::update(&record)
            .set(records::category_id.eq(category.id))
            .execute(db)?;
        //record.set_category(Some(&category));
        //record.save(db)?;

        let merchant = test::merchant(db, "Bar")?;
        diesel::update(&record)
            .set(records::merchant_id.eq(merchant.id))
            .execute(db)?;
        //record.set_merchant(Some(&merchant));
        //record.save(db)?;

        record.reload(db)?;
        assert_eq!(Some(category.id), record.category_id);
        assert_eq!(Some(merchant.id), record.merchant_id);

        Ok(())
    }

    #[test]
    fn clear_merchant_id() -> Result<()> {
        let db = &mut test::db()?;
        let account = test::account(db, "Cash")?;
        let merchant_1 = test::merchant(db, "Foo")?;
        let merchant_2 = test::merchant(db, "Bar")?;

        let mut record_1 = NewRecord {
            merchant: Some(&merchant_1),
            ..NewRecord::new(&account)
        }
        .save(db)?;

        let mut record_2 = NewRecord {
            merchant: Some(&merchant_2),
            ..NewRecord::new(&account)
        }
        .save(db)?;

        super::clear_merchant_id(db, merchant_1.id)?;
        assert_eq!(None, record_1.reload(db)?.merchant_id);
        assert_eq!(Some(merchant_2.id), record_2.reload(db)?.merchant_id);

        Ok(())
    }

    #[test]
    fn clear_category_id() -> Result<()> {
        let db = &mut test::db()?;
        let account = test::account(db, "Cash")?;
        let category_1 = test::category(db, "Foo")?;
        let category_2 = test::category(db, "Bar")?;

        let mut record_1 = NewRecord {
            category: Some(&category_1),
            ..NewRecord::new(&account)
        }
        .save(db)?;

        let mut record_2 = NewRecord {
            category: Some(&category_2),
            ..NewRecord::new(&account)
        }
        .save(db)?;

        super::clear_category_id(db, category_1.id)?;
        assert_eq!(None, record_1.reload(db)?.category_id);
        assert_eq!(Some(category_2.id), record_2.reload(db)?.category_id);

        Ok(())
    }

    #[test]
    fn delete_by_account_id() -> Result<()> {
        let db = &mut test::db()?;
        let account_1 = test::account(db, "Cash")?;
        let account_2 = test::account(db, "Account")?;

        let mut record_1 = test::record(db, &account_1)?;
        let mut record_2 = test::record(db, &account_2)?;

        super::delete_by_account_id(db, account_1.id)?;
        assert!(record_1.reload(db).is_err());
        assert!(record_2.reload(db).is_ok());

        Ok(())
    }
}
