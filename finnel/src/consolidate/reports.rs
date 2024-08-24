use crate::prelude::*;
use crate::schema::{categories, reports_categories};

pub fn consolidate(conn: &mut Conn) -> Result<()> {
    let query = categories::table
        .inner_join(reports_categories::table)
        .filter(categories::replaced_by_id.is_not_null())
        .select(Category::as_select());

    for category in query.load::<Category>(conn)? {
        let old_id = category.id;
        let category = category.resolve(conn)?;

        diesel::update(reports_categories::table)
            .filter(reports_categories::category_id.eq(old_id))
            .set(reports_categories::category_id.eq(category.id))
            .execute(conn)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::ChangeCategory;
    use crate::test::prelude::{assert_eq, Result, *};

    #[test]
    fn consolidate() -> Result<()> {
        let conn = &mut test::db()?;

        let mut report = Report::create(conn, "foo")?;
        let cat1 = test::category!(conn, "cat1");

        report.add(conn, [&cat1])?;

        let cat2 = test::category!(conn, "cat2");
        ChangeCategory {
            replaced_by: Some(Some(&cat2)),
            ..Default::default()
        }
        .save(conn, &cat1)?;

        super::consolidate(conn)?;

        report.reload(conn)?;
        assert_eq!(cat2.id, report.categories[0].id);

        Ok(())
    }
}
