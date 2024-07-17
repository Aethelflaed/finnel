use super::Category;
use crate::essentials::*;
pub use crate::schema::categories;

use diesel::{
    expression::SqlLiteral,
    helper_types::*,
    prelude::*,
    query_source::{Alias, AliasedField},
    sql_types::Text,
    sqlite::Sqlite,
};

diesel::alias! {
    const CATEGORIES_ALIAS: Alias<CategoryAlias> = categories as categories_alias;
    const PARENTS: Alias<Parents> = categories as parents;
    const REPLACERS: Alias<Replacers> = categories as replacers;
}

#[derive(Default)]
pub struct QueryCategory<'a> {
    pub name: Option<&'a str>,
    pub parent_id: Option<Option<i64>>,
    pub count: Option<i64>,
}

pub struct QueryCategoryWithParent<'a>(QueryCategory<'a>);
pub struct QueryCategoryWithReplacer<'a>(QueryCategory<'a>);
pub struct QueryCategoryWithParentAndReplacer<'a>(QueryCategory<'a>);

type CategoryWithParent = (Category, Option<Category>);
type CategoryWithReplacer = (Category, Option<Category>);
type CategoryWithParentAndReplacer =
    (Category, Option<Category>, Option<Category>);

type QueryType<'a> = IntoBoxed<
    'a,
    Filter<
        Alias<CategoryAlias>,
        Like<AliasedField<CategoryAlias, categories::name>, SqlLiteral<Text>>,
    >,
    Sqlite,
>;

impl<'a> QueryCategory<'a> {
    fn build(&self) -> QueryType<'a> {
        let mut query = CATEGORIES_ALIAS.into_boxed();

        if let Some(name) = self.name {
            query = query
                .filter(CATEGORIES_ALIAS.field(categories::name).like(name));
        }
        if let Some(parent_id) = self.parent_id {
            query = query.filter(
                CATEGORIES_ALIAS.field(categories::parent_id).is(parent_id),
            );
        }
        if let Some(count) = self.count {
            query = query.limit(count);
        }

        query
    }

    pub fn run(&self, conn: &mut Conn) -> Result<Vec<Category>> {
        Ok(self
            .build()
            .select(CATEGORIES_ALIAS.fields(categories::all_columns))
            .load::<Category>(conn)?)
    }

    pub fn with_parent(self) -> QueryCategoryWithParent<'a> {
        QueryCategoryWithParent(self)
    }

    pub fn with_replacer(self) -> QueryCategoryWithReplacer<'a> {
        QueryCategoryWithReplacer(self)
    }
}

impl<'a> QueryCategoryWithParent<'a> {
    pub fn run(&self, conn: &mut Conn) -> Result<Vec<CategoryWithParent>> {
        Ok(self
            .0
            .build()
            .left_join(
                PARENTS.on(CATEGORIES_ALIAS
                    .field(categories::parent_id)
                    .eq(PARENTS.field(categories::id).nullable())),
            )
            .select((
                CATEGORIES_ALIAS.fields(categories::all_columns),
                PARENTS.fields(categories::all_columns.nullable()),
            ))
            .load::<CategoryWithParent>(conn)?)
    }

    pub fn with_replacer(self) -> QueryCategoryWithParentAndReplacer<'a> {
        QueryCategoryWithParentAndReplacer(self.0)
    }
}

impl<'a> QueryCategoryWithReplacer<'a> {
    pub fn run(&self, conn: &mut Conn) -> Result<Vec<CategoryWithReplacer>> {
        Ok(self
            .0
            .build()
            .left_join(
                REPLACERS.on(CATEGORIES_ALIAS
                    .field(categories::replaced_by_id)
                    .eq(REPLACERS.field(categories::id).nullable())),
            )
            .select((
                CATEGORIES_ALIAS.fields(categories::all_columns),
                REPLACERS.fields(categories::all_columns.nullable()),
            ))
            .load::<CategoryWithReplacer>(conn)?)
    }

    pub fn with_parent(self) -> QueryCategoryWithParentAndReplacer<'a> {
        QueryCategoryWithParentAndReplacer(self.0)
    }
}

impl QueryCategoryWithParentAndReplacer<'_> {
    pub fn run(
        &self,
        conn: &mut Conn,
    ) -> Result<Vec<CategoryWithParentAndReplacer>> {
        Ok(self
            .0
            .build()
            .left_join(
                PARENTS.on(CATEGORIES_ALIAS
                    .field(categories::parent_id)
                    .eq(PARENTS.field(categories::id).nullable())),
            )
            .left_join(
                REPLACERS.on(CATEGORIES_ALIAS
                    .field(categories::replaced_by_id)
                    .eq(REPLACERS.field(categories::id).nullable())),
            )
            .select((
                CATEGORIES_ALIAS.fields(categories::all_columns),
                PARENTS.fields(categories::all_columns.nullable()),
                REPLACERS.fields(categories::all_columns.nullable()),
            ))
            .load::<CategoryWithParentAndReplacer>(conn)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::category::ChangeCategory;
    use crate::test::prelude::{assert_eq, Result, *};

    #[test]
    fn query() -> Result<()> {
        let conn = &mut test::db()?;
        let category1 = &mut test::category(conn, "Foo")?;
        let category1_1 = &mut test::category(conn, "Bar")?;

        ChangeCategory {
            parent_id: Some(Some(category1.id)),
            replaced_by_id: Some(Some(category1.id)),
            ..Default::default()
        }
        .apply(conn, category1_1)?;

        let result = QueryCategory {
            name: Some("Bar"),
            ..Default::default()
        }
        .with_parent()
        .run(conn)?;
        let Some((_cat, Some(parent))) = result.first() else {
            anyhow::bail!("No result or parent is None");
        };
        assert_eq!(parent.id, category1.id);

        let result = QueryCategory {
            parent_id: Some(Some(parent.id)),
            ..Default::default()
        }
        .with_replacer()
        .with_parent()
        .run(conn)?;

        assert_eq!(1, result.len());
        let Some((_cat, Some(_), Some(rep))) = result.first() else {
            anyhow::bail!("No result or parent is None");
        };
        assert_eq!(rep.id, category1.id);

        Ok(())
    }
}
