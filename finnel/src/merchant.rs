use crate::{category::Category, essentials::*, schema::merchants};

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
}

impl NewMerchant<'_> {
    pub fn save(self, conn: &mut Conn) -> Result<Merchant> {
        Ok(diesel::insert_into(merchants::table)
            .values(self)
            .returning(Merchant::as_returning())
            .get_result(conn)?)
    }
}

pub(crate) fn clear_category_id(conn: &mut Conn, id: i64) -> Result<()> {
    diesel::update(merchants::table)
        .filter(merchants::default_category_id.eq(id))
        .set(merchants::default_category_id.eq(None::<i64>))
        .execute(conn)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::prelude::{assert_eq, Result, *};

    #[test]
    fn create_then_find_by_name() -> Result<()> {
        let conn = &mut test::db()?;

        let merchant = NewMerchant { name: "Bar" }.save(conn)?;

        assert_eq!(
            merchant.id,
            Merchant::find_by_name(conn, &merchant.name)?.id
        );
        assert_eq!(merchant.name, Merchant::find(conn, merchant.id)?.name);

        Ok(())
    }
}
