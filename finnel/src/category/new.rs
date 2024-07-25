use crate::{
    category::Category,
    essentials::*,
    resolved::{mapmap, mapresolve},
    schema::categories,
};

use diesel::prelude::*;

#[derive(Default)]
pub struct NewCategory<'a> {
    pub name: &'a str,
    pub parent: Option<&'a Category>,
    pub replaced_by: Option<&'a Category>,
}

impl<'a> NewCategory<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }

    pub fn save(self, conn: &mut Conn) -> Result<Category> {
        self.to_insertable(conn)?.save(conn)
    }

    pub fn to_insertable(
        self,
        conn: &mut Conn,
    ) -> Result<InsertableCategory<'a>> {
        let NewCategory {
            name,
            parent,
            replaced_by,
        } = self;

        let parent = mapresolve(conn, parent)?;
        let parent_id = mapmap(&parent, |c| c.id);

        let replaced_by = mapresolve(conn, replaced_by)?;
        let replaced_by_id = mapmap(&replaced_by, |c| c.id);

        Ok(InsertableCategory {
            name,
            parent_id,
            replaced_by_id,
        })
    }
}

#[derive(Default, Insertable)]
#[diesel(table_name = categories)]
pub struct InsertableCategory<'a> {
    pub name: &'a str,
    pub parent_id: Option<i64>,
    pub replaced_by_id: Option<i64>,
}

impl<'a> InsertableCategory<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }
}

impl InsertableCategory<'_> {
    pub fn save(self, conn: &mut Conn) -> Result<Category> {
        Ok(diesel::insert_into(categories::table)
            .values(self)
            .returning(Category::as_returning())
            .get_result(conn)?)
    }
}
