use super::Category;

use derive::{Query, QueryDebug};

#[derive(Default, Query, QueryDebug)]
#[query(entity = Category, alias = "categories")]
pub struct QueryCategory {
    #[param(operator = "LIKE")]
    pub name: Option<String>,
    #[param(limit)]
    pub count: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    use anyhow::Result;

    use crate::Database;
    use db::{Entity, Query};

    #[test]
    fn query_merchant() -> Result<()> {
        let db = Database::memory()?;
        db.setup()?;

        let mut category = Category::new("Uraidla Pub");
        category.save(&db)?;

        let query = QueryCategory::default();
        let categories = query
            .statement(&db)?
            .iter()?
            .collect::<rusqlite::Result<Vec<Category>>>()?;

        assert_eq!(categories[0].name, category.name);

        Ok(())
    }
}
