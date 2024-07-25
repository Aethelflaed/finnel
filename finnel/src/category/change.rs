use crate::{
    category::Category,
    essentials::*,
    resolved::{as_resolved, mapmapmap, mapmapmapresult, mapmapresolve},
    schema::categories,
};

use diesel::prelude::*;

pub struct ChangeCategory<'a> {
    pub category: &'a mut Category,
    pub name: Option<&'a str>,
    pub parent: Option<Option<&'a Category>>,
    pub replaced_by: Option<Option<&'a Category>>,
}

fn save_internal(
    conn: &mut Conn,
    category: &Category,
    changeset: CategoryChangeSet,
) -> Result<()> {
    diesel::update(category).set(changeset).execute(conn)?;
    Ok(())
}

impl<'a> ChangeCategory<'a> {
    pub fn new(category: &'a mut Category) -> Self {
        Self {
            category,
            name: None,
            parent: None,
            replaced_by: None,
        }
    }

    pub fn save(self, conn: &mut Conn) -> Result<()> {
        let (category, changeset) = self.to_changeset(conn)?;
        save_internal(conn, category, changeset)
    }

    pub fn apply(self, conn: &mut Conn) -> Result<()> {
        let (category, changeset) = self.to_changeset(conn)?;
        save_internal(conn, category, changeset.clone())?;

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

    fn to_resolved(
        self,
        conn: &mut Conn,
    ) -> Result<ResolvedChangeCategory<'a>> {
        let ChangeCategory {
            name,
            parent,
            replaced_by,
            category,
        } = self;

        Ok(ResolvedChangeCategory {
            name,
            category,
            parent: mapmapresolve(conn, parent)?,
            replaced_by: mapmapresolve(conn, replaced_by)?,
        })
    }

    pub fn to_changeset(
        self,
        conn: &mut Conn,
    ) -> Result<(&'a mut Category, CategoryChangeSet<'a>)> {
        let ResolvedChangeCategory {
            name,
            parent,
            replaced_by,
            category,
        } = self.to_resolved(conn)?.validated(conn)?;

        Ok((
            category,
            CategoryChangeSet {
                name,
                parent_id: mapmapmap(&parent, |c| c.id),
                replaced_by_id: mapmapmap(&replaced_by, |c| c.id),
            },
        ))
    }
}

struct ResolvedChangeCategory<'a> {
    pub category: &'a mut Category,
    pub name: Option<&'a str>,
    pub parent: Option<Option<Resolved<'a, Category>>>,
    pub replaced_by: Option<Option<Resolved<'a, Category>>>,
}

impl<'a> ResolvedChangeCategory<'a> {
    fn validate_parent(&self, conn: &mut Conn) -> Result<()> {
        mapmapmapresult(&self.parent, |parent| {
            if self.category.id == parent.id {
                return Err(Error::Invalid(
                    "category.parent_id should not reference itself".to_owned(),
                ));
            }

            let ancestor =
                as_resolved(conn, parent, Category::find, |c| c.parent_id)?;

            if self.category.id == ancestor.map(|c| c.id) {
                return Err(Error::Invalid(
                    "category.parent_id would create a reference loop"
                        .to_owned(),
                ));
            }

            Ok(())
        })?;
        Ok(())
    }

    fn validate_replace_by(&self, _conn: &mut Conn) -> Result<()> {
        mapmapmapresult(&self.replaced_by, |replaced_by| {
            if self.category.id == replaced_by.id {
                return Err(Error::Invalid(
                    "category.replaced_by_id should not reference itself"
                        .to_owned(),
                ));
            }

            Ok(())
        })?;
        Ok(())
    }

    pub fn validated(self, conn: &mut Conn) -> Result<Self> {
        self.validate_parent(conn)?;
        self.validate_replace_by(conn)?;

        Ok(self)
    }
}

#[derive(Default, Clone, AsChangeset)]
#[diesel(table_name = categories)]
pub struct CategoryChangeSet<'a> {
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
            ..ChangeCategory::new(category1_1)
        }
        .apply(conn)?;

        let change = ChangeCategory {
            parent: Some(Some(category1_1)),
            replaced_by: Some(Some(category1_1)),
            ..ChangeCategory::new(category1)
        };
        let resolved = change.to_resolved(conn)?;

        assert!(resolved.validate_parent(conn).is_err());
        assert!(resolved.validate_replace_by(conn).is_err());

        let change = ChangeCategory {
            parent: Some(Some(category1_1)),
            replaced_by: Some(Some(category1_1)),
            ..ChangeCategory::new(category1)
        };
        assert!(change.save(conn).is_err());

        Ok(())
    }
}
