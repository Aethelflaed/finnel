pub use crate::schema::records;
use crate::{
    account::Account, category::Category, essentials::*, merchant::Merchant,
    Amount, Currency, Decimal,
};

use chrono::{offset::Utc, DateTime};
use diesel::prelude::*;

mod direction;
pub use direction::Direction;

mod mode;
pub use mode::{Mode, PaymentMethod};

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

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = records)]
pub struct NewRecord<'a> {
    pub account_id: i64,
    #[diesel(serialize_as = crate::db::Decimal)]
    pub amount: Decimal,
    #[diesel(serialize_as = crate::db::Currency)]
    pub currency: Currency,
    pub operation_date: DateTime<Utc>,
    pub value_date: DateTime<Utc>,
    pub direction: Direction,
    pub mode: Mode,
    pub details: &'a str,
    pub category_id: Option<i64>,
    pub merchant_id: Option<i64>,
}

impl NewRecord<'_> {
    pub fn new(account: &Account) -> Self {
        Self {
            account_id: account.id,
            currency: account.currency,
            ..Default::default()
        }
    }

    pub fn save(self, conn: &mut Conn) -> Result<Record> {
        Ok(diesel::insert_into(records::table)
            .values(self)
            .returning(Record::as_returning())
            .get_result(conn)?)
    }
}

impl Default for NewRecord<'_> {
    fn default() -> Self {
        let date = Utc::now();

        Self {
            account_id: 0,
            amount: Decimal::ZERO,
            currency: Currency::EUR,
            operation_date: date,
            value_date: date,
            direction: Direction::Debit,
            mode: Mode::Direct(PaymentMethod::Empty),
            details: "",
            category_id: None,
            merchant_id: None,
        }
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

#[derive(Default)]
pub struct QueryRecord<'a> {
    pub account_id: Option<i64>,
    pub after: Option<DateTime<Utc>>,
    pub before: Option<DateTime<Utc>>,
    pub operation_date: bool,
    pub greater_than: Option<Decimal>,
    pub less_than: Option<Decimal>,
    pub direction: Option<Direction>,
    pub mode: Option<Mode>,
    pub merchant_id: Option<Option<i64>>,
    pub category_id: Option<Option<i64>>,
    pub details: Option<&'a str>,
    pub count: Option<i64>,
}

type QueryRecordResult = (Record, Option<Category>, Option<Merchant>);

impl QueryRecord<'_> {
    pub fn run(&self, conn: &mut Conn) -> Result<Vec<QueryRecordResult>> {
        let Some(account_id) = self.account_id else {
            return Err(Error::Invalid("Missing account_id".to_owned()));
        };

        let mut query = records::table
            .into_boxed()
            .filter(records::account_id.eq(account_id));

        if self.operation_date {
            if let Some(date) = self.after {
                query = query.filter(records::operation_date.lt(date));
            }
            if let Some(date) = self.before {
                query = query.filter(records::operation_date.ge(date));
            }
        } else {
            if let Some(date) = self.after {
                query = query.filter(records::value_date.lt(date));
            }
            if let Some(date) = self.before {
                query = query.filter(records::value_date.ge(date));
            }
        }

        if let Some(amount) = self.greater_than {
            query =
                query.filter(records::amount.ge(crate::db::Decimal(amount)));
        }
        if let Some(amount) = self.less_than {
            query =
                query.filter(records::amount.lt(crate::db::Decimal(amount)));
        }
        if let Some(direction) = self.direction {
            query = query.filter(records::direction.eq(direction));
        }
        if let Some(mode) = &self.mode {
            query = query.filter(records::mode.eq(mode));
        }
        if let Some(category_id) = self.category_id {
            query = query.filter(records::category_id.eq(category_id));
        }
        if let Some(merchant_id) = self.merchant_id {
            query = query.filter(records::merchant_id.eq(merchant_id));
        }
        if let Some(details) = self.details {
            query = query.filter(records::details.like(details));
        }

        if let Some(count) = self.count {
            query = query.limit(count);
        }

        Ok(query
            .left_join(crate::schema::categories::table)
            .left_join(crate::schema::merchants::table)
            .select((
                Record::as_select(),
                Option::<Category>::as_select(),
                Option::<Merchant>::as_select(),
            ))
            .load::<QueryRecordResult>(conn)?)
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

        let mut record_1 = NewRecord::new(&account);
        record_1.merchant_id = Some(merchant_1.id);
        let mut record_1 = record_1.save(db)?;

        let mut record_2 = NewRecord::new(&account);
        record_2.merchant_id = Some(merchant_2.id);
        let mut record_2 = record_2.save(db)?;

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

        let mut record_1 = NewRecord::new(&account);
        record_1.category_id = Some(category_1.id);
        let mut record_1 = record_1.save(db)?;

        let mut record_2 = NewRecord::new(&account);
        record_2.category_id = Some(category_2.id);
        let mut record_2 = record_2.save(db)?;

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
