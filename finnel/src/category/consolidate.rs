use super::{query, ChangeCategory};
use crate::prelude::*;
use crate::schema::categories;

pub fn consolidate(conn: &mut Conn) -> Result<()> {
    let query = query::CATEGORIES_ALIAS
        .inner_join(
            query::REPLACERS.on(query::CATEGORIES_ALIAS
                .field(categories::replaced_by_id)
                .eq(query::REPLACERS.field(categories::id).nullable())),
        )
        .filter(
            query::REPLACERS
                .field(categories::replaced_by_id)
                .is_not_null(),
        )
        .select((
            query::CATEGORIES_ALIAS.fields(categories::all_columns),
            query::REPLACERS.fields(categories::all_columns),
        ));

    for (mut category, replacer) in query.load::<(Category, Category)>(conn)? {
        let replacer = replacer.resolve(conn)?;

        ChangeCategory {
            replaced_by: Some(Some(&replacer)),
            ..ChangeCategory::new(&mut category)
        }
        .save(conn)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::category::NewCategory;
    use crate::test::prelude::{assert_eq, Result, *};

    #[test]
    fn consolidate() -> Result<()> {
        let conn = &mut test::db()?;

        let transfer = test::category(conn, "transfer")?;
        let virement = NewCategory {
            name: "virement",
            replaced_by: Some(&transfer),
            ..Default::default()
        }
        .save(conn)?;
        let mut virement_2 = NewCategory {
            name: "virement 2",
            replaced_by: Some(&virement),
            ..Default::default()
        }
        .save(conn)?;

        super::consolidate(conn)?;

        virement_2.reload(conn)?;
        assert_eq!(Some(transfer.id), virement_2.replaced_by_id);

        Ok(())
    }
}
