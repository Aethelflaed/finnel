use super::ChangeRecord;
use crate::prelude::*;
use crate::schema::{categories, merchants, records};

pub fn consolidate(conn: &mut Conn) -> Result<()> {
    consolidate_categories(conn)?;
    consolidate_merchants(conn)?;

    Ok(())
}

pub fn consolidate_categories(conn: &mut Conn) -> Result<()> {
    let query = records::table
        .inner_join(categories::table)
        .filter(categories::replaced_by_id.is_not_null())
        .select((Record::as_select(), Category::as_select()));

    for (record, category) in query.load::<(Record, Category)>(conn)? {
        let category = category.resolve(conn)?;

        ChangeRecord {
            category_id: Some(Some(category.id)),
            ..Default::default()
        }
        .save(conn, &record)?;
    }

    Ok(())
}

pub fn consolidate_merchants(conn: &mut Conn) -> Result<()> {
    let query = records::table
        .inner_join(merchants::table)
        .filter(merchants::replaced_by_id.is_not_null())
        .select((Record::as_select(), Merchant::as_select()));

    for (record, merchant) in query.load::<(Record, Merchant)>(conn)? {
        let merchant = merchant.resolve(conn)?;

        ChangeRecord {
            merchant_id: Some(Some(merchant.id)),
            ..Default::default()
        }
        .save(conn, &record)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::NewCategory;
    use crate::merchant::NewMerchant;
    use crate::record::NewRecord;
    use crate::test::prelude::{assert_eq, Result, *};

    #[test]
    fn consolidate_categories() -> Result<()> {
        let conn = &mut test::db()?;
        let account = test::account(conn, "Cash")?;

        let bar = test::category(conn, "Bar")?;
        let public_house = NewCategory {
            name: "Public House",
            replaced_by: Some(&bar),
            ..Default::default()
        }
        .save(conn)?;

        let mut record = NewRecord {
            details: "beer",
            category_id: Some(public_house.id),
            ..NewRecord::new(&account)
        }
        .save(conn)?;

        consolidate(conn)?;

        record.reload(conn)?;
        assert_eq!(Some(bar.id), record.category_id);

        Ok(())
    }

    #[test]
    fn consolidate_merchants() -> Result<()> {
        let conn = &mut test::db()?;
        let account = test::account(conn, "Cash")?;

        let chariot = test::merchant(conn, "Chariot")?;
        let le_chariot = NewMerchant {
            name: "Le chariot",
            replaced_by: Some(&chariot),
            ..Default::default()
        }
        .save(conn)?;

        let mut record = NewRecord {
            details: "beer",
            merchant_id: Some(le_chariot.id),
            ..NewRecord::new(&account)
        }
        .save(conn)?;

        consolidate(conn)?;

        record.reload(conn)?;
        assert_eq!(Some(chariot.id), record.merchant_id);

        Ok(())
    }
}
