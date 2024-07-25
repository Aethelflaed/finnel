use crate::{essentials::*, schema::categories};

use diesel::prelude::*;

mod new;
pub use new::NewCategory;

mod change;
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

    pub fn resolve(self, conn: &mut Conn) -> Result<Self> {
        if let Some(id) = self.replaced_by_id {
            Self::find(conn, id)?.resolve(conn)
        } else {
            Ok(self)
        }
    }

    pub fn as_resolved(
        &self,
        conn: &mut Conn,
    ) -> Resolved<'_, Category, Error> {
        if let Some(id) = self.replaced_by_id {
            match Category::find(conn, id) {
                Ok(category) => category.resolve(conn).into(),
                Err(e) => Resolved::Err(e),
            }
        } else {
            Resolved::Original(self)
        }
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

    pub fn change(&mut self) -> ChangeCategory<'_> {
        ChangeCategory::new(self)
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
            ..ChangeCategory::new(&mut category)
        }
        .apply(conn)?;
        assert_eq!("Foo", category.name);
        assert_eq!("Foo", category.reload(conn)?.name);

        category.delete(conn)?;
        assert!(Category::find(conn, category.id).is_err());

        Ok(())
    }
}
