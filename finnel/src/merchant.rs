use crate::{category::Category, essentials::*, schema::merchants};

use diesel::prelude::*;

mod new;
pub use new::NewMerchant;

mod change;
pub use change::ChangeMerchant;

mod query;
pub use query::QueryMerchant;

mod consolidate;
pub use consolidate::consolidate;

#[derive(Debug, Queryable, Selectable, Identifiable, Associations)]
#[diesel(table_name = merchants)]
#[diesel(belongs_to(Category, foreign_key = default_category_id))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Merchant {
    pub id: i64,
    pub name: String,
    pub default_category_id: Option<i64>,
    pub replaced_by_id: Option<i64>,
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

impl Resolvable for Merchant {
    fn resolve(self, conn: &mut Conn) -> Result<Self> {
        crate::resolved::resolve(conn, self, Self::find, |c| c.replaced_by_id)
    }

    fn as_resolved<'a>(&'a self, conn: &mut Conn) -> Result<Resolved<'a, Self>> {
        crate::resolved::as_resolved(conn, self, Self::find, |c| c.replaced_by_id)
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
    fn crud() -> Result<()> {
        let conn = &mut test::db()?;

        let merchant = &mut NewMerchant::new("Bar").save(conn)?;

        assert_eq!(
            merchant.id,
            Merchant::find_by_name(conn, &merchant.name)?.id
        );
        assert_eq!(merchant.name, Merchant::find(conn, merchant.id)?.name);

        ChangeMerchant {
            name: Some("Foo"),
            ..Default::default()
        }
        .apply(conn, merchant)?;
        assert_eq!("Foo", merchant.name);
        assert_eq!("Foo", merchant.reload(conn)?.name);

        merchant.delete(conn)?;
        assert!(Merchant::find(conn, merchant.id).is_err());

        Ok(())
    }
}
