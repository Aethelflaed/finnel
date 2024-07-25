use crate::{category::Category, essentials::*, schema::categories};

use diesel::prelude::*;

pub struct ChangeCategory<'a> {
    pub category: &'a mut Category,
    pub name: Option<&'a str>,
    pub parent: Option<Option<&'a Category>>,
    pub replaced_by: Option<Option<&'a Category>>,
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
        changeset.save(conn, &category)
    }

    pub fn apply(self, conn: &mut Conn) -> Result<()> {
        let (mut category, changeset) = self.to_changeset(conn)?;
        changeset.clone().save(conn, &mut category)?;

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

    pub fn to_changeset(
        self,
        conn: &mut Conn,
    ) -> Result<(&'a mut Category, CategoryChangeSet<'a>)> {
        let ChangeCategory {
            name,
            parent,
            replaced_by,
            category,
        } = self;

        let parent_id = parent
            .map(|outer| {
                outer.map(|p| p.as_resolved(conn).map(|p| p.id)).transpose()
            })
            .transpose()?;
        let replaced_by_id = replaced_by
            .map(|outer| {
                outer.map(|p| p.as_resolved(conn).map(|p| p.id)).transpose()
            })
            .transpose()?;

        Ok((
            category,
            CategoryChangeSet {
                name,
                parent_id,
                replaced_by_id,
            },
        ))
    }
}

#[derive(Default, Clone, AsChangeset)]
#[diesel(table_name = categories)]
pub struct CategoryChangeSet<'a> {
    pub name: Option<&'a str>,
    pub parent_id: Option<Option<i64>>,
    pub replaced_by_id: Option<Option<i64>>,
}

fn resolve_self_reference<F>(
    conn: &mut Conn,
    id: i64,
    get_id: F,
) -> Result<Category>
where
    F: Fn(&Category) -> Option<i64>,
{
    let category = Category::find(conn, id)?;
    if let Some(id) = get_id(&category) {
        resolve_self_reference(conn, id, get_id)
    } else {
        Ok(category)
    }
}

impl CategoryChangeSet<'_> {
    fn check_self_reference<F>(
        conn: &mut Conn,
        id: i64,
        ref_id: Option<Option<i64>>,
        get_id: F,
    ) -> Result<()>
    where
        F: Fn(&Category) -> Option<i64>,
    {
        if let Some(Some(ref_id)) = ref_id {
            if id == ref_id {
                return Err(Error::Invalid(
                    "Category references itself".to_owned(),
                ));
            } else if id
                == resolve_self_reference(conn, ref_id, get_id)?.id
            {
                return Err(Error::Invalid(
                    "Reference loop for category".to_owned(),
                ));
            }
        }

        Ok(())
    }

    pub fn valid(&self, conn: &mut Conn, category: &Category) -> Result<()> {
        Self::check_self_reference(conn, category.id, self.parent_id, |c| {
            c.parent_id
        })?;
        Self::check_self_reference(conn, category.id, self.replaced_by_id, |c| {
            c.replaced_by_id
        })?;

        Ok(())
    }

    pub fn save(self, conn: &mut Conn, category: &Category) -> Result<()> {
        self.valid(conn, category)?;
        diesel::update(category).set(self).execute(conn)?;
        Ok(())
    }
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

        let category1_id = category1.id;
        let change = ChangeCategory {
            parent: Some(Some(category1_1)),
            replaced_by: Some(Some(category1_1)),
            ..ChangeCategory::new(category1)
        };

        assert!(CategoryChangeSet::check_self_reference(
            conn,
            category1_id,
            Some(Some(category1_1.id)),
            |c| c.parent_id
        )
        .is_err());
        assert!(CategoryChangeSet::check_self_reference(
            conn,
            category1_id,
            Some(Some(category1_1.id)),
            |c| c.replaced_by_id
        )
        .is_err());
        assert!(change.save(conn).is_err());

        Ok(())
    }
}
