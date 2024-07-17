pub use crate::schema::merchants;
use crate::{category::Category, essentials::*};

use diesel::prelude::*;

mod query;
pub use query::QueryMerchant;

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

    pub fn resolve(self, conn: &mut Conn) -> Result<Self> {
        if let Some(id) = self.replaced_by_id {
            Self::find(conn, id)?.resolve(conn)
        } else {
            Ok(self)
        }
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

#[derive(Default, Insertable)]
#[diesel(table_name = merchants)]
pub struct NewMerchant<'a> {
    pub name: &'a str,
    pub default_category_id: Option<i64>,
    pub replaced_by_id: Option<i64>,
}

impl<'a> NewMerchant<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            ..Default::default()
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

#[derive(Default, Clone, AsChangeset)]
#[diesel(table_name = merchants)]
pub struct ChangeMerchant<'a> {
    pub name: Option<&'a str>,
    pub default_category_id: Option<Option<i64>>,
    pub replaced_by_id: Option<Option<i64>>,
}

impl ChangeMerchant<'_> {
    fn check_self_reference(
        conn: &mut Conn,
        id: Option<Option<i64>>,
        merchant: &Merchant,
    ) -> Result<()> {
        if let Some(Some(id)) = id {
            if merchant.id == id {
                return Err(Error::Invalid(
                    "Merchant references itself".to_owned(),
                ));
            } else if merchant.id == Merchant::find(conn, id)?.resolve(conn)?.id
            {
                return Err(Error::Invalid(
                    "Reference loop for merchant".to_owned(),
                ));
            }
        }

        Ok(())
    }

    pub fn valid(&self, conn: &mut Conn, merchant: &Merchant) -> Result<()> {
        Self::check_self_reference(conn, self.replaced_by_id, merchant)?;

        Ok(())
    }

    pub fn save(self, conn: &mut Conn, merchant: &Merchant) -> Result<()> {
        self.valid(conn, merchant)?;
        diesel::update(merchant).set(self).execute(conn)?;
        Ok(())
    }

    pub fn apply(self, conn: &mut Conn, merchant: &mut Merchant) -> Result<()> {
        self.clone().save(conn, merchant)?;

        if let Some(value) = self.name {
            merchant.name = value.to_string();
        }
        if let Some(value) = self.default_category_id {
            merchant.default_category_id = value;
        }
        if let Some(value) = self.replaced_by_id {
            merchant.replaced_by_id = value;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::prelude::{assert_eq, Result, *};

    #[test]
    fn crud() -> Result<()> {
        let conn = &mut test::db()?;

        let mut merchant = NewMerchant::new("Bar").save(conn)?;

        assert_eq!(
            merchant.id,
            Merchant::find_by_name(conn, &merchant.name)?.id
        );
        assert_eq!(merchant.name, Merchant::find(conn, merchant.id)?.name);

        ChangeMerchant {
            name: Some("Foo"),
            ..Default::default()
        }
        .apply(conn, &mut merchant)?;
        assert_eq!("Foo", merchant.name);
        assert_eq!("Foo", merchant.reload(conn)?.name);

        merchant.delete(conn)?;
        assert!(Merchant::find(conn, merchant.id).is_err());

        Ok(())
    }

    #[test]
    fn update_loop() -> Result<()> {
        let conn = &mut test::db()?;
        let merchant1 = &mut test::merchant(conn, "Foo")?;
        let merchant1_1 = &mut test::merchant(conn, "Bar")?;

        ChangeMerchant {
            replaced_by_id: Some(Some(merchant1.id)),
            ..Default::default()
        }
        .apply(conn, merchant1_1)?;

        let change = ChangeMerchant {
            replaced_by_id: Some(Some(merchant1_1.id)),
            ..Default::default()
        };

        assert!(ChangeMerchant::check_self_reference(
            conn,
            change.replaced_by_id,
            merchant1
        )
        .is_err());
        assert!(change.clone().save(conn, merchant1).is_err());
        assert!(change.clone().save(conn, merchant1_1).is_err());

        Ok(())
    }
}
