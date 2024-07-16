use crate::essentials::*;
pub use crate::schema::categories;

use diesel::prelude::*;

mod query;
pub use query::QueryCategory;

#[derive(Debug, Queryable, Selectable, Identifiable, Associations)]
#[diesel(table_name = categories)]
#[diesel(belongs_to(Category, foreign_key = parent_id))]
//#[diesel(belongs_to(Category, foreign_key = replaced_by_id))]
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
    pub parent_id: Option<Option<i64>>,
    pub replaced_by_id: Option<Option<i64>>,
}

impl ChangeCategory<'_> {
    fn check_loop(
        conn: &mut Conn,
        id: Option<Option<i64>>,
        category: &Category,
    ) -> Result<()> {
        if let Some(Some(id)) = id {
            if category.id == id {
                return Err(Error::Invalid(
                    "Category references itself".to_owned(),
                ));
            } else if category.id == Category::find(conn, id)?.resolve(conn)?.id
            {
                return Err(Error::Invalid(
                    "Reference loop for category".to_owned(),
                ));
            }
        }

        Ok(())
    }

    pub fn valid(&self, conn: &mut Conn, category: &Category) -> Result<()> {
        Self::check_loop(conn, self.parent_id, category)?;
        Self::check_loop(conn, self.replaced_by_id, category)?;

        Ok(())
    }

    pub fn save(&self, conn: &mut Conn, category: &Category) -> Result<()> {
        self.valid(conn, category)?;
        diesel::update(category).set(self).execute(conn)?;
        Ok(())
    }

    pub fn apply(self, conn: &mut Conn, category: &mut Category) -> Result<()> {
        self.save(conn, category)?;

        if let Some(value) = self.name {
            category.name = value.to_string();
        }
        if let Some(value) = self.parent_id {
            category.parent_id = value;
        }
        if let Some(value) = self.replaced_by_id {
            category.replaced_by_id = value;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::prelude::{assert_eq, Result, *};

    #[test]
    fn crud() -> Result<()> {
        let conn = &mut test::db()?;

        let mut category = NewCategory { name: "Bar" }.save(conn)?;

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

    #[test]
    fn update_loop() -> Result<()> {
        let conn = &mut test::db()?;
        let category1 = &mut test::category(conn, "Foo")?;
        let category1_1 = &mut test::category(conn, "Bar")?;

        ChangeCategory {
            parent_id: Some(Some(category1.id)),
            replaced_by_id: Some(Some(category1.id)),
            ..Default::default()
        }
        .apply(conn, category1_1)?;

        let change = ChangeCategory {
            parent_id: Some(Some(category1_1.id)),
            replaced_by_id: Some(Some(category1_1.id)),
            ..Default::default()
        };

        assert!(
            ChangeCategory::check_loop(conn, change.parent_id, category1)
                .is_err()
        );
        assert!(ChangeCategory::check_loop(
            conn,
            change.replaced_by_id,
            category1
        )
        .is_err());
        assert!(change.save(conn, category1).is_err());
        assert!(change.save(conn, category1_1).is_err());

        Ok(())
    }
}
