use crate::{
    prelude::*,
    resolved::{mapmapmap, mapmapresolve},
    schema::recurring_payments,
};
use diesel::prelude::*;

#[derive(Default, Clone)]
pub struct ChangeRecurringPayment<'a> {
    pub name: Option<&'a str>,
    pub description: Option<&'a str>,
    pub frequency: Option<Frequency>,
    pub account: Option<&'a Account>,
    pub amount: Option<Decimal>,
    pub direction: Option<Direction>,
    pub mode: Option<Mode>,
    pub category: Option<Option<&'a Category>>,
    pub merchant: Option<Option<&'a Merchant>>,
}

impl<'a> ChangeRecurringPayment<'a> {
    pub fn save(self, conn: &mut Conn, recpay: &RecurringPayment) -> Result<()> {
        self.into_resolved(conn)?.validate(conn, recpay)?.save(conn)
    }

    pub fn into_resolved(self, conn: &mut Conn) -> Result<ResolvedChangeRecurringPayment<'a>> {
        Ok(ResolvedChangeRecurringPayment {
            name: self.name,
            description: self.description,
            frequency: self.frequency,
            account: self.account,
            amount: self.amount,
            direction: self.direction,
            mode: self.mode,
            category: mapmapresolve(conn, self.category)?,
            merchant: mapmapresolve(conn, self.merchant)?,
        })
    }
}

pub struct ResolvedChangeRecurringPayment<'a> {
    pub name: Option<&'a str>,
    pub description: Option<&'a str>,
    pub frequency: Option<Frequency>,
    pub account: Option<&'a Account>,
    pub amount: Option<Decimal>,
    pub direction: Option<Direction>,
    pub mode: Option<Mode>,
    pub category: Option<Option<Resolved<'a, Category>>>,
    pub merchant: Option<Option<Resolved<'a, Merchant>>>,
}

impl<'a> ResolvedChangeRecurringPayment<'a> {
    pub fn validate(&self, _conn: &mut Conn, recpay: &'a RecurringPayment) -> Result<ValidatedChangeRecurringPayment<'a>> {
        // Is there anything to do ?

        Ok(ValidatedChangeRecurringPayment(recpay, self.as_changeset()))
    }

    pub fn as_changeset(&self) -> RecurringPaymentChangeset<'a> {
        RecurringPaymentChangeset {
            name: self.name,
            description: self.description,
            frequency: self.frequency,
            account_id: self.account.map(|a| a.id),
            amount: self.amount,
            direction: self.direction,
            mode: self.mode,
            category_id: mapmapmap(&self.category, |c| c.id),
            merchant_id: mapmapmap(&self.merchant, |m| m.id),
        }
    }
}

pub struct ValidatedChangeRecurringPayment<'a>(&'a RecurringPayment, RecurringPaymentChangeset<'a>);

impl<'a> ValidatedChangeRecurringPayment<'a> {
    pub fn save(self, conn: &mut Conn) -> Result<()> {
        diesel::update(self.0).set(self.1).execute(conn)?;
        Ok(())
    }
}

#[derive(Default, Clone, AsChangeset)]
#[diesel(table_name = recurring_payments)]
pub struct RecurringPaymentChangeset<'a> {
    pub name: Option<&'a str>,
    pub description: Option<&'a str>,
    pub frequency: Option<Frequency>,
    pub account_id: Option<i64>,
    #[diesel(serialize_as = crate::db::Decimal)]
    pub amount: Option<Decimal>,
    pub direction: Option<Direction>,
    pub mode: Option<Mode>,
    pub category_id: Option<Option<i64>>,
    pub merchant_id: Option<Option<i64>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::prelude::{*, assert_eq, Result};

    #[test]
    fn change() -> Result<()> {
        let conn = &mut test::db()?;
        let account = test::account!(conn, "Cash");
        let mut recpay = test::recpay!(conn, &account);

        let change = ChangeRecurringPayment {
            name: Some("Foo"),
            ..Default::default()
        };
        change.save(conn, &recpay)?;
        recpay.reload(conn)?;
        assert_eq!("Foo", recpay.name.as_str());

        Ok(())
    }
}
