use crate::prelude::*;
use crate::schema::recurring_payments;
use diesel::prelude::*;

pub mod frequency;
pub use frequency::Frequency;

pub mod new;
pub use new::NewRecurringPayment;

#[derive(Debug, Queryable, Selectable, Identifiable, Associations)]
#[diesel(table_name = recurring_payments)]
#[diesel(belongs_to(Account, foreign_key = account_id))]
#[diesel(belongs_to(Category, foreign_key = category_id))]
#[diesel(belongs_to(Merchant, foreign_key = merchant_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct RecurringPayment {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub frequency: Frequency,
    pub account_id: i64,
    #[diesel(deserialize_as = crate::db::Decimal)]
    pub amount: Decimal,
    #[diesel(deserialize_as = crate::db::Currency)]
    pub currency: Currency,
    pub direction: Direction,
    pub mode: Mode,
    pub category_id: Option<i64>,
    pub merchant_id: Option<i64>,
}

impl RecurringPayment {
    pub fn find(conn: &mut Conn, id: i64) -> Result<Self> {
        recurring_payments::table
            .find(id)
            .select(RecurringPayment::as_select())
            .first(conn)
            .map_err(|e| Error::from_diesel_error(e, "RecurringPayment", None))
    }

    pub fn find_by_name(conn: &mut Conn, name: &str) -> Result<Self> {
        recurring_payments::table
            .filter(recurring_payments::name.eq(name))
            .select(RecurringPayment::as_select())
            .first(conn)
            .map_err(|e| Error::from_diesel_error(e, "RecurringPayment", Some("name")))
    }

    pub fn delete(&mut self, conn: &mut Conn) -> Result<()> {
        diesel::delete(&*self).execute(conn)?;

        Ok(())
    }
}

pub(crate) fn clear_category_id(conn: &mut Conn, id: i64) -> Result<()> {
    diesel::update(recurring_payments::table)
        .filter(recurring_payments::category_id.eq(id))
        .set(recurring_payments::category_id.eq(None::<i64>))
        .execute(conn)?;
    Ok(())
}

pub(crate) fn clear_merchant_id(conn: &mut Conn, id: i64) -> Result<()> {
    diesel::update(recurring_payments::table)
        .filter(recurring_payments::merchant_id.eq(id))
        .set(recurring_payments::merchant_id.eq(None::<i64>))
        .execute(conn)?;
    Ok(())
}

pub(crate) fn delete_by_account_id(conn: &mut Conn, id: i64) -> Result<()> {
    diesel::delete(recurring_payments::table)
        .filter(recurring_payments::account_id.eq(id))
        .execute(conn)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::prelude::{assert_eq, Result, *};

    #[test]
    fn clear_merchant_id() -> Result<()> {
        let conn = &mut test::db()?;
        let account = test::account!(conn, "Cash");
        let merchant_1 = test::merchant!(conn, "Foo");
        let merchant_2 = test::merchant!(conn, "Bar");

        let mut recpay_1 = NewRecurringPayment {
            merchant: Some(&merchant_1),
            ..NewRecurringPayment::new(&account)
        }
        .save(conn)?;

        let mut recpay_2 = NewRecurringPayment {
            merchant: Some(&merchant_2),
            ..NewRecurringPayment::new(&account)
        }
        .save(conn)?;

        super::clear_merchant_id(conn, merchant_1.id)?;
        assert_eq!(None, recpay_1.reload(conn)?.merchant_id);
        assert_eq!(Some(merchant_2.id), recpay_2.reload(conn)?.merchant_id);

        Ok(())
    }

    #[test]
    fn clear_category_id() -> Result<()> {
        let conn = &mut test::db()?;
        let account = test::account!(conn, "Cash");
        let category_1 = test::category!(conn, "Foo");
        let category_2 = test::category!(conn, "Bar");

        let mut recpay_1 = NewRecurringPayment {
            category: Some(&category_1),
            ..NewRecurringPayment::new(&account)
        }
        .save(conn)?;

        let mut recpay_2 = NewRecurringPayment {
            category: Some(&category_2),
            ..NewRecurringPayment::new(&account)
        }
        .save(conn)?;

        super::clear_category_id(conn, category_1.id)?;
        assert_eq!(None, recpay_1.reload(conn)?.category_id);
        assert_eq!(Some(category_2.id), recpay_2.reload(conn)?.category_id);

        Ok(())
    }

    #[test]
    fn delete_by_account_id() -> Result<()> {
        let conn = &mut test::db()?;
        let account_1 = test::account!(conn, "Cash");
        let account_2 = test::account!(conn, "Account");

        let mut recpay_1 = NewRecurringPayment::new(&account_1).save(conn)?;
        let mut recpay_2 = NewRecurringPayment::new(&account_2).save(conn)?;

        super::delete_by_account_id(conn, account_1.id)?;
        assert!(recpay_1.reload(conn).is_err());
        assert!(recpay_2.reload(conn).is_ok());

        Ok(())
    }
}
