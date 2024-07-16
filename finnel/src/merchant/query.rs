use super::Merchant;
use crate::category::Category;
use crate::essentials::*;
pub use crate::schema::{categories, merchants};

use diesel::{
    expression::SqlLiteral,
    helper_types::*,
    prelude::*,
    query_source::{Alias, AliasedField},
    sql_types::Text,
    sqlite::Sqlite,
};

diesel::alias! {
    const MERCHANTS_ALIAS: Alias<MerchantAlias> = merchants as merchants_alias;
    const REPLACERS: Alias<Replacers> = merchants as replacers;
}

#[derive(Default)]
pub struct QueryMerchant<'a> {
    pub name: Option<&'a str>,
    pub count: Option<i64>,
}

pub struct QueryMerchantWithCategory<'a>(QueryMerchant<'a>);
pub struct QueryMerchantWithReplacer<'a>(QueryMerchant<'a>);
pub struct QueryMerchantWithCategoryAndReplacer<'a>(QueryMerchant<'a>);

type MerchantWithCategory = (Merchant, Option<Category>);
type MerchantWithReplacer = (Merchant, Option<Merchant>);
type MerchantWithCategoryAndReplacer =
    (Merchant, Option<Category>, Option<Merchant>);

type QueryType<'a> = IntoBoxed<
    'a,
    Filter<
        Alias<MerchantAlias>,
        Like<AliasedField<MerchantAlias, merchants::name>, SqlLiteral<Text>>,
    >,
    Sqlite,
>;

impl<'a> QueryMerchant<'a> {
    fn build(&self) -> QueryType<'a> {
        let mut query = MERCHANTS_ALIAS.into_boxed();

        if let Some(name) = self.name {
            query =
                query.filter(MERCHANTS_ALIAS.field(merchants::name).like(name));
        }
        if let Some(count) = self.count {
            query = query.limit(count);
        }

        query
    }

    pub fn run(&self, conn: &mut Conn) -> Result<Vec<Merchant>> {
        Ok(self
            .build()
            .select(MERCHANTS_ALIAS.fields(merchants::all_columns))
            .load::<Merchant>(conn)?)
    }

    pub fn with_category(self) -> QueryMerchantWithCategory<'a> {
        QueryMerchantWithCategory(self)
    }

    pub fn with_replacer(self) -> QueryMerchantWithReplacer<'a> {
        QueryMerchantWithReplacer(self)
    }
}

impl<'a> QueryMerchantWithCategory<'a> {
    pub fn run(&self, conn: &mut Conn) -> Result<Vec<MerchantWithCategory>> {
        Ok(self
            .0
            .build()
            .left_join(
                categories::table.on(MERCHANTS_ALIAS
                    .field(merchants::default_category_id)
                    .eq(categories::id.nullable())),
            )
            .select((
                MERCHANTS_ALIAS.fields(merchants::all_columns),
                categories::all_columns.nullable(),
            ))
            .load::<MerchantWithCategory>(conn)?)
    }

    pub fn with_replacer(self) -> QueryMerchantWithCategoryAndReplacer<'a> {
        QueryMerchantWithCategoryAndReplacer(self.0)
    }
}

impl<'a> QueryMerchantWithReplacer<'a> {
    pub fn run(&self, conn: &mut Conn) -> Result<Vec<MerchantWithReplacer>> {
        Ok(self
            .0
            .build()
            .left_join(
                REPLACERS.on(MERCHANTS_ALIAS
                    .field(merchants::replaced_by_id)
                    .eq(REPLACERS.field(merchants::id).nullable())),
            )
            .select((
                MERCHANTS_ALIAS.fields(merchants::all_columns),
                REPLACERS.fields(merchants::all_columns.nullable()),
            ))
            .load::<MerchantWithReplacer>(conn)?)
    }

    pub fn with_category(self) -> QueryMerchantWithCategoryAndReplacer<'a> {
        QueryMerchantWithCategoryAndReplacer(self.0)
    }
}

impl QueryMerchantWithCategoryAndReplacer<'_> {
    pub fn run(
        &self,
        conn: &mut Conn,
    ) -> Result<Vec<MerchantWithCategoryAndReplacer>> {
        Ok(self
            .0
            .build()
            .left_join(
                categories::table.on(MERCHANTS_ALIAS
                    .field(merchants::default_category_id)
                    .eq(categories::id.nullable())),
            )
            .left_join(
                REPLACERS.on(MERCHANTS_ALIAS
                    .field(merchants::replaced_by_id)
                    .eq(REPLACERS.field(merchants::id).nullable())),
            )
            .select((
                MERCHANTS_ALIAS.fields(merchants::all_columns),
                categories::all_columns.nullable(),
                REPLACERS.fields(merchants::all_columns.nullable()),
            ))
            .load::<MerchantWithCategoryAndReplacer>(conn)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::merchant::ChangeMerchant;
    use crate::test::prelude::{assert_eq, Result, *};

    #[test]
    fn query() -> Result<()> {
        let conn = &mut test::db()?;
        let merchant1 = &mut test::merchant(conn, "Foo")?;
        let merchant1_1 = &mut test::merchant(conn, "Bar")?;
        let category = &test::category(conn, "Bar")?;

        ChangeMerchant {
            default_category_id: Some(Some(category.id)),
            replaced_by_id: Some(Some(merchant1.id)),
            ..Default::default()
        }
        .apply(conn, merchant1_1)?;

        let result = QueryMerchant {
            name: Some("Bar"),
            ..Default::default()
        }
        .with_replacer()
        .run(conn)?;
        let Some((_cat, Some(rep))) = result.first() else {
            anyhow::bail!("No result or replacer is None");
        };
        assert_eq!(rep.id, merchant1.id);

        let result = QueryMerchant {
            name: Some("Bar"),
            ..Default::default()
        }
        .with_category()
        .run(conn)?;

        assert_eq!(1, result.len());
        let Some((_cat, Some(cat))) = result.first() else {
            anyhow::bail!("No result or category is None");
        };
        assert_eq!(cat.id, category.id);

        Ok(())
    }
}
