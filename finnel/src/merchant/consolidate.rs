use super::ChangeMerchant;
use crate::prelude::*;
use crate::schema::{self, categories, merchants};

pub fn consolidate(conn: &mut Conn) -> Result<()> {
    consolidate_replace_by(conn)?;
    consolidate_default_category(conn)?;

    Ok(())
}

pub fn consolidate_replace_by(conn: &mut Conn) -> Result<()> {
    let (merchants, replacers) = diesel::alias!(
        schema::merchants as merchants,
        schema::merchants as replacers
    );

    let query = merchants
        .inner_join(
            replacers.on(merchants
                .field(merchants::replaced_by_id)
                .eq(replacers.field(merchants::id).nullable())),
        )
        .filter(replacers.field(merchants::replaced_by_id).is_not_null())
        .select((
            merchants.fields(merchants::all_columns),
            replacers.fields(merchants::all_columns),
        ));

    for (merchant, replacer) in query.load::<(Merchant, Merchant)>(conn)? {
        let replacer = replacer.resolve(conn)?;

        ChangeMerchant {
            replaced_by: Some(Some(&replacer)),
            ..Default::default()
        }
        .save(conn, &merchant)?;
    }

    Ok(())
}

pub fn consolidate_default_category(conn: &mut Conn) -> Result<()> {
    let query = merchants::table
        .inner_join(categories::table)
        .filter(categories::replaced_by_id.is_not_null())
        .select((merchants::all_columns, categories::all_columns));

    for (merchant, category) in query.load::<(Merchant, Category)>(conn)? {
        let category = category.resolve(conn)?;

        ChangeMerchant {
            default_category: Some(Some(&category)),
            ..Default::default()
        }
        .save(conn, &merchant)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::merchant::NewMerchant;
    use crate::test::prelude::{assert_eq, Result, *};

    #[test]
    fn consolidate_replace_by() -> Result<()> {
        let conn = &mut test::db()?;

        let chariot = NewMerchant {
            name: "chariot",
            ..Default::default()
        }
        .save(conn)?;

        let le_chariot = NewMerchant {
            name: "le chariot",
            replaced_by: Some(&chariot),
            ..Default::default()
        }
        .save(conn)?;
        let mut bar_le_chariot = NewMerchant {
            name: "bar le chariot",
            replaced_by: Some(&le_chariot),
            ..Default::default()
        }
        .save(conn)?;

        consolidate(conn)?;

        bar_le_chariot.reload(conn)?;
        assert_eq!(Some(chariot.id), bar_le_chariot.replaced_by_id);

        Ok(())
    }

    #[test]
    fn consolidate_default_category() -> Result<()> {
        let conn = &mut test::db()?;

        let bar = test::category(conn, "bar")?;
        let mut chariot = NewMerchant {
            name: "Chariot",
            default_category: Some(&bar),
            ..Default::default()
        }
        .save(conn)?;

        let capital_bar = test::category(conn, "Bar")?;
        crate::category::ChangeCategory {
            replaced_by: Some(Some(&capital_bar)),
            ..Default::default()
        }
        .save(conn, &bar)?;

        consolidate(conn)?;

        chariot.reload(conn)?;
        assert_eq!(Some(capital_bar.id), chariot.default_category_id);

        Ok(())
    }
}
