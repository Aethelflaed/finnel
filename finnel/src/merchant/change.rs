use crate::{
    category::Category,
    merchant::Merchant,
    essentials::*,
    resolved::{mapmapmap, mapmapmapresult, mapmapresolve},
    schema::merchants,
};

use diesel::prelude::*;

pub struct ChangeMerchant<'a> {
    pub merchant: &'a mut Merchant,
    pub name: Option<&'a str>,
    pub default_category: Option<Option<&'a Category>>,
    pub replaced_by: Option<Option<&'a Merchant>>,
}

fn save_internal(
    conn: &mut Conn,
    merchant: &Merchant,
    changeset: MerchantChangeset,
) -> Result<()> {
    diesel::update(merchant).set(changeset).execute(conn)?;
    Ok(())
}

impl<'a> ChangeMerchant<'a> {
    pub fn new(merchant: &'a mut Merchant) -> Self {
        Self {
            merchant,
            name: None,
            default_category: None,
            replaced_by: None,
        }
    }

    pub fn save(self, conn: &mut Conn) -> Result<()> {
        let (merchant, changeset) = self.to_changeset(conn)?;
        save_internal(conn, merchant, changeset)
    }

    pub fn apply(self, conn: &mut Conn) -> Result<()> {
        let (merchant, changeset) = self.to_changeset(conn)?;
        save_internal(conn, merchant, changeset.clone())?;

        if let Some(value) = changeset.name {
            merchant.name = value.to_string();
        }
        if let Some(value) = changeset.default_category_id {
            merchant.default_category_id = value;
        }
        if let Some(value) = changeset.replaced_by_id {
            merchant.replaced_by_id = value;
        }

        Ok(())
    }

    pub fn to_resolved(
        self,
        conn: &mut Conn,
    ) -> Result<ResolvedChangeMerchant<'a>> {
        let ChangeMerchant {
            name,
            default_category,
            replaced_by,
            merchant,
        } = self;

        Ok(ResolvedChangeMerchant {
            name,
            merchant,
            default_category: mapmapresolve(conn, default_category)?,
            replaced_by: mapmapresolve(conn, replaced_by)?,
        })
    }

    pub fn to_changeset(
        self,
        conn: &mut Conn,
    ) -> Result<(&'a mut Merchant, MerchantChangeset<'a>)> {
        let ResolvedChangeMerchant {
            name,
            default_category,
            replaced_by,
            merchant,
        } = self.to_resolved(conn)?.validated(conn)?;

        Ok((
            merchant,
            MerchantChangeset {
                name,
                default_category_id: mapmapmap(&default_category, |c| c.id),
                replaced_by_id: mapmapmap(&replaced_by, |m| m.id),
            },
        ))
    }
}

pub struct ResolvedChangeMerchant<'a> {
    pub merchant: &'a mut Merchant,
    pub name: Option<&'a str>,
    pub default_category: Option<Option<Resolved<'a, Category>>>,
    pub replaced_by: Option<Option<Resolved<'a, Merchant>>>,
}

impl<'a> ResolvedChangeMerchant<'a> {
    fn validate_replace_by(&self, _conn: &mut Conn) -> Result<()> {
        mapmapmapresult(&self.replaced_by, |replaced_by| {
            if self.merchant.id == replaced_by.id {
                return Err(Error::Invalid(
                    "merchant.replaced_by_id should not reference itself"
                        .to_owned(),
                ));
            }

            Ok(())
        })?;
        Ok(())
    }

    pub fn validated(self, conn: &mut Conn) -> Result<Self> {
        self.validate_replace_by(conn)?;

        Ok(self)
    }
}

#[derive(Default, Clone, AsChangeset)]
#[diesel(table_name = merchants)]
pub struct MerchantChangeset<'a> {
    pub name: Option<&'a str>,
    pub default_category_id: Option<Option<i64>>,
    pub replaced_by_id: Option<Option<i64>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::prelude::{Result, *};

    #[test]
    fn update_loop() -> Result<()> {
        let conn = &mut test::db()?;
        let merchant1 = &mut test::merchant(conn, "Foo")?;
        let merchant1_1 = &mut test::merchant(conn, "Bar")?;

        ChangeMerchant {
            replaced_by: Some(Some(merchant1)),
            ..ChangeMerchant::new(merchant1_1)
        }
        .apply(conn)?;

        let change = ChangeMerchant {
            replaced_by: Some(Some(merchant1_1)),
            ..ChangeMerchant::new(merchant1)
        };
        let resolved = change.to_resolved(conn)?;

        assert!(resolved.validate_replace_by(conn).is_err());

        let change = ChangeMerchant {
            replaced_by: Some(Some(merchant1_1)),
            ..ChangeMerchant::new(merchant1)
        };
        assert!(change.save(conn).is_err());

        Ok(())
    }
}
