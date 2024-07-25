use crate::{
    prelude::*,
    resolved::{mapmap, mapresolve},
    schema::records,
};

use chrono::{offset::Utc, DateTime};
use diesel::prelude::*;

pub struct NewRecord<'a> {
    pub account: &'a Account,
    pub amount: Decimal,
    pub currency: Currency,
    pub operation_date: DateTime<Utc>,
    pub value_date: DateTime<Utc>,
    pub direction: Direction,
    pub mode: Mode,
    pub details: &'a str,
    pub category: Option<&'a Category>,
    pub merchant: Option<&'a Merchant>,
}

impl<'a> NewRecord<'a> {
    pub fn new(account: &'a Account) -> Self {
        let date = Utc::now();

        Self {
            account,
            amount: Decimal::ZERO,
            currency: account.currency,
            operation_date: date,
            value_date: date,
            direction: Direction::Debit,
            mode: Mode::Direct(PaymentMethod::Empty),
            details: "",
            category: None,
            merchant: None,
        }
    }

    pub fn save(self, conn: &mut Conn) -> Result<Record> {
        self.to_resolved(conn)?.validate(conn)?.save(conn)
    }

    pub fn to_resolved(self, conn: &mut Conn) -> Result<ResolvedNewRecord<'a>> {
        Ok(ResolvedNewRecord {
            account: self.account,
            amount: self.amount,
            currency: self.currency,
            operation_date: self.operation_date,
            value_date: self.value_date,
            direction: self.direction,
            mode: self.mode,
            details: self.details,
            category: mapresolve(conn, self.category)?,
            merchant: mapresolve(conn, self.merchant)?,
        })
    }
}

pub struct ResolvedNewRecord<'a> {
    pub account: &'a Account,
    pub amount: Decimal,
    pub currency: Currency,
    pub operation_date: DateTime<Utc>,
    pub value_date: DateTime<Utc>,
    pub direction: Direction,
    pub mode: Mode,
    pub details: &'a str,
    pub category: Option<Resolved<'a, Category>>,
    pub merchant: Option<Resolved<'a, Merchant>>,
}

impl<'a> ResolvedNewRecord<'a> {
    pub fn validate(self, _conn: &mut Conn) -> Result<ValidatedNewRecord<'a>> {
        if self.currency != self.account.currency {
            return Err(Error::Invalid(
                "record.currency should match the account currency".to_owned(),
            ));
        }

        Ok(ValidatedNewRecord(self.as_insertable()))
    }

    pub fn as_insertable(&self) -> InsertableRecord<'a> {
        InsertableRecord {
            account_id: self.account.id,
            amount: self.amount,
            currency: self.currency,
            operation_date: self.operation_date,
            value_date: self.value_date,
            direction: self.direction,
            mode: self.mode,
            details: self.details,
            category_id: mapmap(&self.category, |c| c.id),
            merchant_id: mapmap(&self.merchant, |m| m.id),
        }
    }
}

pub struct ValidatedNewRecord<'a>(InsertableRecord<'a>);

impl<'a> ValidatedNewRecord<'a> {
    pub fn save(self, conn: &mut Conn) -> Result<Record> {
        Ok(diesel::insert_into(records::table)
            .values(self.0)
            .returning(Record::as_returning())
            .get_result(conn)?)
    }
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = records)]
pub struct InsertableRecord<'a> {
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

impl InsertableRecord<'_> {
    pub fn save(self, conn: &mut Conn) -> Result<Record> {
        Ok(diesel::insert_into(records::table)
            .values(self)
            .returning(Record::as_returning())
            .get_result(conn)?)
    }
}
