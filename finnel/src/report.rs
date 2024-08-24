use crate::{
    category::Category,
    essentials::*,
    schema::{categories, reports, reports_categories},
};

use diesel::prelude::*;

pub struct Report {
    pub id: i64,
    pub name: String,
    pub categories: Vec<Category>,
}

impl Report {
    pub fn create(conn: &mut Conn, name: &str) -> Result<Self> {
        diesel::insert_into(reports::table)
            .values(reports::name.eq(name))
            .returning((reports::id, reports::name))
            .get_result(conn)
            .map_err(|e| Error::from_diesel_error(e, "Report", None))
            .and_then(|(id, name)| Self::load(conn, id, name))
    }

    pub fn find(conn: &mut Conn, id: i64) -> Result<Self> {
        reports::table
            .find(id)
            .select((reports::id, reports::name))
            .first(conn)
            .map_err(|e| Error::from_diesel_error(e, "Report", None))
            .and_then(|(id, name)| Self::load(conn, id, name))
    }

    pub fn find_by_name(conn: &mut Conn, name: &str) -> Result<Self> {
        reports::table
            .filter(reports::name.eq(name))
            .select((reports::id, reports::name))
            .first(conn)
            .map_err(|e| Error::from_diesel_error(e, "Report", Some("name")))
            .and_then(|(id, name)| Self::load(conn, id, name))
    }

    pub fn all(conn: &mut Conn) -> Result<Vec<(i64, String)>> {
        Ok(reports::table
            .select((reports::id, reports::name))
            .load(conn)?)
    }

    pub fn add<'a, T>(&mut self, conn: &mut Conn, iter: T) -> Result<()>
    where
        T: IntoIterator<Item = &'a Category>,
    {
        let values = iter
            .into_iter()
            .map(|c| {
                Ok((
                    reports_categories::report_id.eq(self.id),
                    reports_categories::category_id.eq(c.as_resolved(conn)?.map(|c| c.id)),
                ))
            })
            .collect::<Result<Vec<_>>>()?;

        diesel::insert_into(reports_categories::table)
            .values(values)
            .execute(conn)?;

        Ok(())
    }

    pub fn remove<'a, T>(&mut self, conn: &mut Conn, iter: T) -> Result<()>
    where
        T: IntoIterator<Item = &'a Category>,
    {
        let values = iter
            .into_iter()
            .map(|c| Ok(c.as_resolved(conn)?.map(|c| c.id)))
            .collect::<Result<Vec<_>>>()?;
        diesel::delete(reports_categories::table)
            .filter(reports_categories::report_id.eq(self.id))
            .filter(reports_categories::category_id.eq_any(values))
            .execute(conn)?;
        Ok(())
    }

    pub fn delete(&mut self, conn: &mut Conn) -> Result<()> {
        diesel::delete(reports_categories::table)
            .filter(reports_categories::report_id.eq(self.id))
            .execute(conn)?;
        diesel::delete(reports::table)
            .filter(reports::id.eq(self.id))
            .execute(conn)?;
        Ok(())
    }

    fn load(conn: &mut Conn, id: i64, name: String) -> Result<Self> {
        Ok(Report {
            id,
            name,
            categories: categories::table
                .inner_join(reports_categories::table)
                .filter(reports_categories::report_id.eq(id))
                .select(Category::as_select())
                .load::<Category>(conn)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::prelude::{assert_eq, Result, *};
    use diesel::dsl::count_star;

    #[test]
    fn test() -> Result<()> {
        let conn = &mut test::db()?;

        let mut report = Report::create(conn, "foo")?;

        let real_cat1 = &test::category!(conn, "real cat1");
        let cat1 = &test::category!(conn, "cat1", replaced_by: Some(&real_cat1));
        let cat2 = &test::category!(conn, "cat2");
        let cat3 = &test::category!(conn, "cat3");

        report.add(conn, [cat2, cat1])?;
        assert!(report.add(conn, [cat1]).is_err());

        let mut report = Report::find_by_name(conn, "foo")?;

        assert_eq!(2, report.categories.len());
        let mut ids = report.categories.iter().map(|c| c.id).collect::<Vec<_>>();
        ids.sort();
        assert_eq!(vec![real_cat1.id, cat2.id], ids);

        report.remove(conn, [cat3])?;
        report.remove(conn, [cat1])?;

        report.reload(conn)?;

        assert_eq!(1, report.categories.len());
        assert_eq!(cat2.id, report.categories[0].id);

        assert_eq!(report.id, Report::find_by_name(conn, "foo")?.id);

        report.delete(conn)?;

        assert_eq!(0i64, reports::table.select(count_star()).first(conn)?);
        assert_eq!(
            0i64,
            reports_categories::table.select(count_star()).first(conn)?
        );

        Ok(())
    }
}
