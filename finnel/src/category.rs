use crate::{essentials::*, schema::categories};

use diesel::prelude::*;

pub mod new;
pub use new::NewCategory;

pub mod change;
pub use change::ChangeCategory;

mod query;
pub use query::QueryCategory;

mod consolidate;
pub use consolidate::consolidate;

#[derive(Debug, Clone, Queryable, Selectable, Identifiable)]
#[diesel(table_name = categories)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub replaced_by_id: Option<i64>,
}

impl Category {
    pub fn find(conn: &mut Conn, id: i64) -> Result<Self> {
        categories::table
            .find(id)
            .select(Category::as_select())
            .first(conn)
            .map_err(|e| e.into())
    }

    pub fn find_by_name(conn: &mut Conn, name: &str) -> Result<Self> {
        categories::table
            .filter(categories::name.eq(name))
            .select(Category::as_select())
            .first(conn)
            .map_err(|e| e.into())
    }

    /// Delete the current category, nulling references to it where possible
    ///
    /// This method executes multiple queries without wrapping them in a
    /// transaction
    pub fn delete(&mut self, conn: &mut Conn) -> Result<()> {
        crate::record::clear_category_id(conn, self.id)?;
        crate::merchant::clear_category_id(conn, self.id)?;
        diesel::delete(&*self).execute(conn)?;

        Ok(())
    }
}

impl Resolvable for Category {
    fn resolve(self, conn: &mut Conn) -> Result<Self> {
        crate::resolved::resolve(conn, self, Self::find, |c| c.replaced_by_id)
    }

    fn as_resolved<'a>(&'a self, conn: &mut Conn) -> Result<Resolved<'a, Self>> {
        crate::resolved::as_resolved(conn, self, Self::find, |c| c.replaced_by_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::prelude::{assert_eq, Result, *};

    #[test]
    fn crud() -> Result<()> {
        let conn = &mut test::db()?;

        let mut category = NewCategory::new("Bar").save(conn)?;

        assert_eq!(
            category.id,
            Category::find_by_name(conn, &category.name)?.id
        );
        assert_eq!(category.name, Category::find(conn, category.id)?.name);

        ChangeCategory {
            name: Some("Foo"),
            ..Default::default()
        }
        .apply(conn, &mut category)?;
        assert_eq!("Foo", category.name);
        assert_eq!("Foo", category.reload(conn)?.name);

        category.delete(conn)?;
        assert!(Category::find(conn, category.id).is_err());

        Ok(())
    }
}
