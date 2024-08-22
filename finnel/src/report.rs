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
