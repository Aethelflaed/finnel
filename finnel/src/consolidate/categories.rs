use crate::category::ChangeCategory;
use crate::prelude::*;
use crate::schema::{self, categories};

pub fn consolidate(conn: &mut Conn) -> Result<()> {
    consolidate_replace_by(conn)?;
    consolidate_parent(conn)?;

    Ok(())
}

pub fn consolidate_replace_by(conn: &mut Conn) -> Result<()> {
    let (categories, replacers) = diesel::alias!(
        schema::categories as categories,
        schema::categories as replacers
    );

    let query = categories
        .inner_join(
            replacers.on(categories
                .field(categories::replaced_by_id)
                .eq(replacers.field(categories::id).nullable())),
        )
        .filter(replacers.field(categories::replaced_by_id).is_not_null())
        .select((
            categories.fields(categories::all_columns),
            replacers.fields(categories::all_columns),
        ));

    for (category, replacer) in query.load::<(Category, Category)>(conn)? {
        let replacer = replacer.resolve(conn)?;

        ChangeCategory {
            replaced_by: Some(Some(&replacer)),
            ..Default::default()
        }
        .save(conn, &category)?;
    }

    Ok(())
}

pub fn consolidate_parent(conn: &mut Conn) -> Result<()> {
    let (categories, parents) = diesel::alias!(
        schema::categories as categories,
        schema::categories as parents
    );

    let query = categories
        .inner_join(
            parents.on(categories
                .field(categories::parent_id)
                .eq(parents.field(categories::id).nullable())),
        )
        .filter(parents.field(categories::replaced_by_id).is_not_null())
        .select((
            categories.fields(categories::all_columns),
            parents.fields(categories::all_columns),
        ));

    for (category, parent) in query.load::<(Category, Category)>(conn)? {
        let parent = parent.resolve(conn)?;

        ChangeCategory {
            parent: Some(Some(&parent)),
            ..Default::default()
        }
        .save(conn, &category)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::NewCategory;
    use crate::test::prelude::{assert_eq, Result, *};

    #[test]
    fn consolidate_replace_by() -> Result<()> {
        let conn = &mut test::db()?;

        let transfer = test::category!(conn, "transfer");
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

    #[test]
    fn consolidate_parent() -> Result<()> {
        let conn = &mut test::db()?;

        let mut alcool = test::category!(conn, "alcool");
        let mut bar = NewCategory {
            name: "bar",
            parent: Some(&alcool),
            ..NewCategory::default()
        }
        .save(conn)?;

        let alcohol = test::category!(conn, "alcohol");
        ChangeCategory {
            replaced_by: Some(Some(&alcohol)),
            ..ChangeCategory::default()
        }
        .apply(conn, &mut alcool)?;

        super::consolidate(conn)?;

        bar.reload(conn)?;
        assert_eq!(Some(alcohol.id), bar.parent_id);

        Ok(())
    }
}
