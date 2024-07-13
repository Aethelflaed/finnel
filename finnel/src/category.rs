use crate::essentials::*;
pub use crate::schema::categories;

use diesel::prelude::*;

#[derive(Debug, Queryable, Selectable, Identifiable)]
#[diesel(table_name = categories)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Category {
    pub id: i64,
    pub name: String,
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

#[derive(Insertable)]
#[diesel(table_name = categories)]
pub struct NewCategory<'a> {
    pub name: &'a str,
}

impl<'a> NewCategory<'a> {
    pub fn new(name: &'a str) -> Self {
        Self { name }
    }
}

impl NewCategory<'_> {
    pub fn save(self, conn: &mut Conn) -> Result<Category> {
        Ok(diesel::insert_into(categories::table)
            .values(self)
            .returning(Category::as_returning())
            .get_result(conn)?)
    }
}

#[derive(Default, Clone, Copy, AsChangeset)]
#[diesel(table_name = categories)]
pub struct ChangeCategory<'a> {
    pub name: Option<&'a str>,
}

impl ChangeCategory<'_> {
    pub fn save(&self, conn: &mut Conn, category: &Category) -> Result<()> {
        diesel::update(category).set(self).execute(conn)?;
        Ok(())
    }

    pub fn apply(self, conn: &mut Conn, category: &mut Category) -> Result<()> {
        self.save(conn, category)?;

        if let Some(value) = self.name {
            category.name = value.to_string();
        }

        Ok(())
    }
}

#[derive(Default)]
pub struct QueryCategory<'a> {
    pub name: Option<&'a str>,
    pub count: Option<i64>,
}

impl QueryCategory<'_> {
    pub fn run(&self, conn: &mut Conn) -> Result<Vec<Category>> {
        let mut query = categories::table.into_boxed();

        if let Some(name) = self.name {
            query = query.filter(categories::name.like(name));
        }
        if let Some(count) = self.count {
            query = query.limit(count);
        }

        Ok(query.select(Category::as_select()).load(conn)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::prelude::{assert_eq, Result, *};

    #[test]
    fn create_then_find_by_name() -> Result<()> {
        let conn = &mut test::db()?;

        let category = NewCategory { name: "Bar" }.save(conn)?;

        assert_eq!(
            category.id,
            Category::find_by_name(conn, &category.name)?.id
        );
        assert_eq!(category.name, Category::find(conn, category.id)?.name);

        Ok(())
    }
}
