use super::{query, ChangeMerchant};
use crate::prelude::*;
use crate::schema::merchants;

pub fn consolidate(conn: &mut Conn) -> Result<()> {
    let query = query::MERCHANTS_ALIAS
        .inner_join(
            query::REPLACERS.on(query::MERCHANTS_ALIAS
                .field(merchants::replaced_by_id)
                .eq(query::REPLACERS.field(merchants::id).nullable())),
        )
        .filter(
            query::REPLACERS
                .field(merchants::replaced_by_id)
                .is_not_null(),
        )
        .select((
            query::MERCHANTS_ALIAS.fields(merchants::all_columns),
            query::REPLACERS.fields(merchants::all_columns),
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

#[cfg(test)]
mod tests {
    use crate::merchant::NewMerchant;
    use crate::test::prelude::{assert_eq, Result, *};

    #[test]
    fn consolidate() -> Result<()> {
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

        super::consolidate(conn)?;

        bar_le_chariot.reload(conn)?;
        assert_eq!(Some(chariot.id), bar_le_chariot.replaced_by_id);

        Ok(())
    }
}
