use crate::{
    category::Category,
    essentials::*,
    merchant::Merchant,
    resolved::{mapmapmap, mapmapmapresult, mapmapresolve},
    schema::merchants,
};

use diesel::prelude::*;

#[derive(Default, Clone)]
pub struct ChangeMerchant<'a> {
    pub name: Option<&'a str>,
    pub default_category: Option<Option<&'a Category>>,
    pub replaced_by: Option<Option<&'a Merchant>>,
}

impl<'a> ChangeMerchant<'a> {
    pub fn save(self, conn: &mut Conn, merchant: &Merchant) -> Result<()> {
        self.into_resolved(conn)?
            .validate(conn, merchant)?
            .save(conn)
    }

    pub fn apply(self, conn: &mut Conn, merchant: &mut Merchant) -> Result<()> {
        let resolved = self.into_resolved(conn)?;
        let changeset = resolved.as_changeset();
        resolved.validate(conn, merchant)?.save(conn)?;

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

    pub fn into_resolved(self, conn: &mut Conn) -> Result<ResolvedChangeMerchant<'a>> {
        Ok(ResolvedChangeMerchant {
            name: self.name,
            default_category: mapmapresolve(conn, self.default_category)?,
            replaced_by: mapmapresolve(conn, self.replaced_by)?,
        })
    }
}

pub struct ResolvedChangeMerchant<'a> {
    name: Option<&'a str>,
    default_category: Option<Option<Resolved<'a, Category>>>,
    replaced_by: Option<Option<Resolved<'a, Merchant>>>,
}

impl<'a> ResolvedChangeMerchant<'a> {
    fn validate_replace_by(&self, _conn: &mut Conn, merchant: &Merchant) -> Result<()> {
        mapmapmapresult(&self.replaced_by, |replaced_by| {
            if merchant.id == replaced_by.id {
                return Err(Error::Invalid(
                    "merchant.replaced_by_id should not reference itself".to_owned(),
                ));
            }

            Ok(())
        })?;
        Ok(())
    }

    pub fn validate(
        &self,
        conn: &mut Conn,
        merchant: &'a Merchant,
    ) -> Result<ValidatedChangeMerchant<'a>> {
        self.validate_replace_by(conn, merchant)?;

        Ok(ValidatedChangeMerchant(merchant, self.as_changeset()))
    }

    pub fn as_changeset(&self) -> MerchantChangeset<'a> {
        MerchantChangeset {
            name: self.name,
            default_category_id: mapmapmap(&self.default_category, |c| c.id),
            replaced_by_id: mapmapmap(&self.replaced_by, |m| m.id),
        }
    }
}

pub struct ValidatedChangeMerchant<'a>(&'a Merchant, MerchantChangeset<'a>);

impl<'a> ValidatedChangeMerchant<'a> {
    pub fn save(self, conn: &mut Conn) -> Result<()> {
        diesel::update(self.0).set(self.1).execute(conn)?;
        Ok(())
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
            ..Default::default()
        }
        .apply(conn, merchant1_1)?;

        let change = ChangeMerchant {
            replaced_by: Some(Some(merchant1_1)),
            ..Default::default()
        };
        let resolved = change.clone().into_resolved(conn)?;

        assert!(resolved.validate_replace_by(conn, merchant1).is_err());

        assert!(change.save(conn, merchant1).is_err());

        Ok(())
    }
}
