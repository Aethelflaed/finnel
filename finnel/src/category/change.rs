use crate::{
    category::Category,
    essentials::*,
    resolved::{as_resolved, mapmapmap, mapmapmapresult, mapmapresolve},
    schema::categories,
};

use diesel::prelude::*;

#[derive(Default, Clone)]
pub struct ChangeCategory<'a> {
    pub name: Option<&'a str>,
    pub parent: Option<Option<&'a Category>>,
    pub replaced_by: Option<Option<&'a Category>>,
}

impl<'a> ChangeCategory<'a> {
    pub fn save(self, conn: &mut Conn, category: &Category) -> Result<()> {
        self.to_resolved(conn)?.validate(conn, category)?.save(conn)
    }

    pub fn apply(self, conn: &mut Conn, category: &mut Category) -> Result<()> {
        let resolved = self.to_resolved(conn)?;
        let changeset = resolved.as_changeset();
        resolved.validate(conn, category)?.save(conn)?;

        if let Some(value) = changeset.name {
            category.name = value.to_string();
        }
        if let Some(value) = changeset.parent_id {
            category.parent_id = value;
        }
        if let Some(value) = changeset.replaced_by_id {
            category.replaced_by_id = value;
        }

        Ok(())
    }

    pub fn to_resolved(self, conn: &mut Conn) -> Result<ResolvedChangeCategory<'a>> {
        Ok(ResolvedChangeCategory {
            name: self.name,
            parent: mapmapresolve(conn, self.parent)?,
            replaced_by: mapmapresolve(conn, self.replaced_by)?,
        })
    }
}

pub struct ResolvedChangeCategory<'a> {
    name: Option<&'a str>,
    parent: Option<Option<Resolved<'a, Category>>>,
    replaced_by: Option<Option<Resolved<'a, Category>>>,
}

impl<'a> ResolvedChangeCategory<'a> {
    fn validate_parent(&self, conn: &mut Conn, category: &Category) -> Result<()> {
        mapmapmapresult(&self.parent, |parent| {
            if category.id == parent.id {
                return Err(Error::Invalid(
                    "category.parent_id should not reference itself".to_owned(),
                ));
            }

            let ancestor = as_resolved(conn, parent, Category::find, |c| c.parent_id)?;

            if category.id == ancestor.map(|c| c.id) {
                return Err(Error::Invalid(
                    "category.parent_id would create a reference loop".to_owned(),
                ));
            }

            Ok(())
        })?;
        Ok(())
    }

    fn validate_replace_by(&self, _conn: &mut Conn, category: &Category) -> Result<()> {
        mapmapmapresult(&self.replaced_by, |replaced_by| {
            if category.id == replaced_by.id {
                return Err(Error::Invalid(
                    "category.replaced_by_id should not reference itself".to_owned(),
                ));
            }

            Ok(())
        })?;
        Ok(())
    }

    pub fn validate(
        self,
        conn: &mut Conn,
        category: &'a Category,
    ) -> Result<ValidatedChangeCategory<'a>> {
        self.validate_parent(conn, category)?;
        self.validate_replace_by(conn, category)?;

        Ok(ValidatedChangeCategory(category, self.as_changeset()))
    }

    pub fn as_changeset(&self) -> CategoryChangeset<'a> {
        CategoryChangeset {
            name: self.name,
            parent_id: mapmapmap(&self.parent, |c| c.id),
            replaced_by_id: mapmapmap(&self.replaced_by, |c| c.id),
        }
    }
}

pub struct ValidatedChangeCategory<'a>(&'a Category, CategoryChangeset<'a>);

impl<'a> ValidatedChangeCategory<'a> {
    pub fn save(self, conn: &mut Conn) -> Result<()> {
        diesel::update(self.0).set(self.1).execute(conn)?;
        Ok(())
    }
}

#[derive(Default, Clone, AsChangeset)]
#[diesel(table_name = categories)]
pub struct CategoryChangeset<'a> {
    pub name: Option<&'a str>,
    pub parent_id: Option<Option<i64>>,
    pub replaced_by_id: Option<Option<i64>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::prelude::{Result, *};

    #[test]
    fn update_loop() -> Result<()> {
        let conn = &mut test::db()?;
        let category1 = &mut test::category(conn, "Foo")?;
        let category1_1 = &mut test::category(conn, "Bar")?;

        ChangeCategory {
            parent: Some(Some(category1)),
            replaced_by: Some(Some(category1)),
            ..Default::default()
        }
        .apply(conn, category1_1)?;

        let change = ChangeCategory {
            parent: Some(Some(category1_1)),
            replaced_by: Some(Some(category1_1)),
            ..Default::default()
        };
        let resolved = change.clone().to_resolved(conn)?;

        assert!(resolved.validate_parent(conn, category1).is_err());
        assert!(resolved.validate_replace_by(conn, category1).is_err());

        assert!(change.save(conn, category1).is_err());

        Ok(())
    }
}
