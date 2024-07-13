pub use crate::schema::merchants;
use crate::{category::Category, essentials::*};

use diesel::prelude::*;

#[derive(Debug, Queryable, Selectable, Identifiable, Associations)]
#[diesel(table_name = merchants)]
#[diesel(belongs_to(Category, foreign_key = default_category_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Merchant {
    pub id: i64,
    pub name: String,
    pub default_category_id: Option<i64>,
}

impl Merchant {
    pub fn find(conn: &mut Conn, id: i64) -> Result<Self> {
        merchants::table
            .find(id)
            .select(Merchant::as_select())
            .first(conn)
            .map_err(|e| e.into())
    }

    pub fn find_by_name(conn: &mut Conn, name: &str) -> Result<Self> {
        merchants::table
            .filter(merchants::name.eq(name))
            .select(Merchant::as_select())
            .first(conn)
            .map_err(|e| e.into())
    }

    /// Delete the current merchant, nulling references to it where possible
    ///
    /// This method executes multiple queries without wrapping them in a
    /// transaction
    pub fn delete(&mut self, conn: &mut Conn) -> Result<()> {
        crate::record::clear_merchant_id(conn, self.id)?;
        diesel::delete(&*self).execute(conn)?;

        Ok(())
    }
}

#[derive(Insertable)]
#[diesel(table_name = merchants)]
pub struct NewMerchant<'a> {
    pub name: &'a str,
    pub default_category_id: Option<i64>,
}

impl<'a> NewMerchant<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            default_category_id: None,
        }
    }
}

impl NewMerchant<'_> {
    pub fn save(self, conn: &mut Conn) -> Result<Merchant> {
        Ok(diesel::insert_into(merchants::table)
            .values(self)
            .returning(Merchant::as_returning())
            .get_result(conn)?)
    }
}

#[derive(Default, Clone, Copy, AsChangeset)]
#[diesel(table_name = merchants)]
pub struct ChangeMerchant<'a> {
    pub name: Option<&'a str>,
    pub default_category_id: Option<Option<i64>>,
}

impl ChangeMerchant<'_> {
    pub fn save(&self, conn: &mut Conn, merchant: &Merchant) -> Result<()> {
        diesel::update(merchant).set(self).execute(conn)?;
        Ok(())
    }

    pub fn apply(self, conn: &mut Conn, merchant: &mut Merchant) -> Result<()> {
        self.save(conn, merchant)?;

        if let Some(value) = self.name {
            merchant.name = value.to_string();
        }
        if let Some(value) = self.default_category_id {
            merchant.default_category_id = value;
        }

        Ok(())
    }
}

pub(crate) fn clear_category_id(conn: &mut Conn, id: i64) -> Result<()> {
    diesel::update(merchants::table)
        .filter(merchants::default_category_id.eq(id))
        .set(merchants::default_category_id.eq(None::<i64>))
        .execute(conn)?;

    Ok(())
}

#[derive(Default)]
pub struct QueryMerchant<'a> {
    pub name: Option<&'a str>,
    pub count: Option<i64>,
}

impl QueryMerchant<'_> {
    pub fn run(&self, conn: &mut Conn) -> Result<Vec<Merchant>> {
        let mut query = merchants::table.into_boxed();

        if let Some(name) = self.name {
            query = query.filter(merchants::name.like(name));
        }
        if let Some(count) = self.count {
            query = query.limit(count);
        }

        Ok(query.select(Merchant::as_select()).load(conn)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::prelude::{assert_eq, Result, *};

    #[test]
    fn create_then_find_by_name() -> Result<()> {
        let conn = &mut test::db()?;

        let merchant = NewMerchant::new("Bar").save(conn)?;

        assert_eq!(
            merchant.id,
            Merchant::find_by_name(conn, &merchant.name)?.id
        );
        assert_eq!(merchant.name, Merchant::find(conn, merchant.id)?.name);

        Ok(())
    }
}
