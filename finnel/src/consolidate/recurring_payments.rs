use crate::prelude::*;
use crate::recurring_payment::ChangeRecurringPayment;
use crate::schema::{recurring_payments, categories, merchants};

pub fn consolidate(conn: &mut Conn) -> Result<()> {
    consolidate_categories(conn)?;
    consolidate_merchants(conn)?;

    Ok(())
}

pub fn consolidate_categories(conn: &mut Conn) -> Result<()> {
    let query = recurring_payments::table
        .inner_join(categories::table)
        .filter(categories::replaced_by_id.is_not_null())
        .select((RecurringPayment::as_select(), Category::as_select()));

    for (recpay, category) in query.load::<(RecurringPayment, Category)>(conn)? {
        let category = category.resolve(conn)?;

        ChangeRecurringPayment {
            category: Some(Some(&category)),
            ..Default::default()
        }
        .save(conn, &recpay)?;
    }

    Ok(())
}

pub fn consolidate_merchants(conn: &mut Conn) -> Result<()> {
    let query = recurring_payments::table
        .inner_join(merchants::table)
        .filter(merchants::replaced_by_id.is_not_null())
        .select((RecurringPayment::as_select(), Merchant::as_select()));

    for (recpay, merchant) in query.load::<(RecurringPayment, Merchant)>(conn)? {
        let merchant = merchant.resolve(conn)?;

        ChangeRecurringPayment {
            merchant: Some(Some(&merchant)),
            ..Default::default()
        }
        .save(conn, &recpay)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::NewCategory;
    use crate::merchant::NewMerchant;
    use crate::recurring_payment::NewRecurringPayment;
    use crate::test::prelude::{assert_eq, Result, *};

    #[test]
    fn consolidate_categories() -> Result<()> {
        let conn = &mut test::db()?;
        let account = test::account!(conn, "Cash");

        let bar = test::category!(conn, "Bar");
        let public_house = NewCategory {
            name: "Public House",
            replaced_by: Some(&bar),
            ..Default::default()
        }
        .save(conn)?;

        let mut recpay = NewRecurringPayment {
            name: "beer",
            category: Some(&public_house),
            ..NewRecurringPayment::new(&account)
        }
        .save(conn)?;

        consolidate(conn)?;

        recpay.reload(conn)?;
        assert_eq!(Some(bar.id), recpay.category_id);

        Ok(())
    }

    #[test]
    fn consolidate_merchants() -> Result<()> {
        let conn = &mut test::db()?;
        let account = test::account!(conn, "Cash");

        let chariot = test::merchant!(conn, "Chariot");
        let le_chariot = NewMerchant {
            name: "Le chariot",
            replaced_by: Some(&chariot),
            ..Default::default()
        }
        .save(conn)?;

        let mut recpay = NewRecurringPayment {
            name: "beer",
            merchant: Some(&le_chariot),
            ..NewRecurringPayment::new(&account)
        }
        .save(conn)?;

        consolidate(conn)?;

        recpay.reload(conn)?;
        assert_eq!(Some(chariot.id), recpay.merchant_id);

        Ok(())
    }
}
