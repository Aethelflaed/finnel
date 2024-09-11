use crate::{
    prelude::*,
    resolved::{mapmap, mapresolve},
    schema::recurring_payments,
};

use diesel::prelude::*;

pub struct NewRecurringPayment<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub frequency: Frequency,
    pub account: &'a Account,
    pub amount: Decimal,
    pub direction: Direction,
    pub mode: Mode,
    pub category: Option<&'a Category>,
    pub merchant: Option<&'a Merchant>,
}

impl<'a> NewRecurringPayment<'a> {
    pub fn new(account: &'a Account) -> Self {
        Self {
            name: "",
            description: "",
            frequency: Frequency::default(),
            account,
            amount: Decimal::ZERO,
            direction: Direction::default(),
            mode: Mode::default(),
            category: None,
            merchant: None,
        }
    }

    pub fn save(self, conn: &mut Conn) -> Result<RecurringPayment> {
        self.into_resolved(conn)?.validate(conn)?.save(conn)
    }

    pub fn into_resolved(self, conn: &mut Conn) -> Result<ResolvedNewRecurringPayment<'a>> {
        Ok(ResolvedNewRecurringPayment {
            name: self.name,
            description: self.description,
            frequency: self.frequency,
            account: self.account,
            amount: self.amount,
            direction: self.direction,
            mode: self.mode,
            category: mapresolve(conn, self.category)?,
            merchant: mapresolve(conn, self.merchant)?,
        })
    }
}

pub struct ResolvedNewRecurringPayment<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub frequency: Frequency,
    pub account: &'a Account,
    pub amount: Decimal,
    pub direction: Direction,
    pub mode: Mode,
    pub category: Option<Resolved<'a, Category>>,
    pub merchant: Option<Resolved<'a, Merchant>>,
}

impl<'a> ResolvedNewRecurringPayment<'a> {
    pub fn validate(&self, _conn: &mut Conn) -> Result<ValidatedNewRecurringPayment<'a>> {
        Ok(ValidatedNewRecurringPayment(self.as_insertable()))
    }

    pub fn as_insertable(&self) -> InsertableRecurringPayment<'a> {
        InsertableRecurringPayment {
            name: self.name,
            description: self.description,
            frequency: self.frequency,
            account_id: self.account.id,
            amount: self.amount,
            currency: self.account.currency,
            direction: self.direction,
            mode: self.mode,
            category_id: mapmap(&self.category, |c| c.id),
            merchant_id: mapmap(&self.merchant, |m| m.id),
        }
    }
}

pub struct ValidatedNewRecurringPayment<'a>(InsertableRecurringPayment<'a>);

impl<'a> ValidatedNewRecurringPayment<'a> {
    pub fn save(self, conn: &mut Conn) -> Result<RecurringPayment> {
        Ok(diesel::insert_into(recurring_payments::table)
            .values(self.0)
            .returning(RecurringPayment::as_returning())
            .get_result(conn)?)
    }
}

#[derive(Debug, Clone, Insertable)]
#[diesel(table_name = recurring_payments)]
pub struct InsertableRecurringPayment<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub frequency: Frequency,
    pub account_id: i64,
    #[diesel(serialize_as = db::Decimal)]
    pub amount: Decimal,
    #[diesel(serialize_as = db::Currency)]
    pub currency: Currency,
    pub direction: Direction,
    pub mode: Mode,
    pub category_id: Option<i64>,
    pub merchant_id: Option<i64>,
}
