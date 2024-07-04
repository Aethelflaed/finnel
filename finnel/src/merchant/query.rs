use super::Merchant;

use derive::{Query, QueryDebug};

#[derive(Default, Query, QueryDebug)]
#[query(entity = Merchant, alias = "merchants")]
pub struct QueryMerchant {
    #[param(operator = "LIKE")]
    pub details: Option<String>,
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

        let mut merchant = Merchant::new("Uraidla Pub");
        merchant.save(&db)?;

        let query = QueryMerchant::default();
        let merchants = query
            .statement(&db)?
            .iter()?
            .collect::<rusqlite::Result<Vec<Merchant>>>()?;

        assert_eq!(&merchants[0].name, merchant.name());

        Ok(())
    }
}
