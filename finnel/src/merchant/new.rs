use crate::{
    category::Category,
    essentials::*,
    merchant::Merchant,
    resolved::{mapmap, mapresolve},
    schema::merchants,
};

use diesel::prelude::*;

#[derive(Default)]
pub struct NewMerchant<'a> {
    pub name: &'a str,
    pub default_category: Option<&'a Category>,
    pub replaced_by: Option<&'a Merchant>,
}

impl<'a> NewMerchant<'a> {
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }

    pub fn save(self, conn: &mut Conn) -> Result<Merchant> {
        self.to_insertable(conn)?.save(conn)
    }

    pub fn to_insertable(
        self,
        conn: &mut Conn,
    ) -> Result<InsertableMerchant<'a>> {
        let NewMerchant {
            name,
            default_category,
            replaced_by,
        } = self;

        let default_category = mapresolve(conn, default_category)?;
        let replaced_by = mapresolve(conn, replaced_by)?;

        Ok(InsertableMerchant {
            name,
            default_category_id: mapmap(&default_category, |c| c.id),
            replaced_by_id: mapmap(&replaced_by, |m| m.id),
        })
    }
}

#[derive(Default, Insertable)]
#[diesel(table_name = merchants)]
pub struct InsertableMerchant<'a> {
    pub name: &'a str,
    pub default_category_id: Option<i64>,
    pub replaced_by_id: Option<i64>,
}

impl InsertableMerchant<'_> {
    pub fn save(self, conn: &mut Conn) -> Result<Merchant> {
        Ok(diesel::insert_into(merchants::table)
            .values(self)
            .returning(Merchant::as_returning())
            .get_result(conn)?)
    }
}
