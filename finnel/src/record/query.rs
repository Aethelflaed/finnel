use chrono::{offset::Utc, DateTime};

use crate::transaction::{Direction, Mode};
use crate::{Category, Merchant, Record};
use db::{Decimal, Id, Row};

use derive::{Query, QueryDebug};

pub struct FullRecord {
    pub record: Record,
    pub merchant: Option<Merchant>,
    pub category: Option<Category>,
}

impl TryFrom<Row<'_>> for FullRecord {
    type Error = rusqlite::Error;

    fn try_from(row: Row) -> rusqlite::Result<Self> {
        Self::try_from(&row)
    }
}
impl TryFrom<&Row<'_>> for FullRecord {
    type Error = rusqlite::Error;

    fn try_from(row: &Row) -> rusqlite::Result<Self> {
        let record =
            row.with_prefix("records_", |row| Record::try_from(row))?;
        let merchant =
            row.with_prefix("merchants_", |row| Merchant::try_from(row).ok());
        let category =
            row.with_prefix("categories_", |row| Category::try_from(row).ok());

        Ok(FullRecord {
            record,
            merchant,
            category,
        })
    }
}

#[derive(Default, Query, QueryDebug)]
#[query(result = FullRecord, entity = Record, alias = "records")]
#[join(
    type = "LEFT",
    lhs(field = "merchant_id", entity = Record, alias = "records"),
    rhs(field = "id", entity = Merchant, alias = "merchants")
)]
#[join(
    type = "LEFT",
    lhs(field = "category_id", entity = Record, alias = "records"),
    rhs(field = "id", entity = Category, alias = "categories")
)]
pub struct QueryRecord {
    #[param(mandatory)]
    pub account_id: Option<Id>,
    #[param(
        field =
            if self.operation_date {
                "operation_date"
            } else {
                "value_date"
            },
        operator = ">=",
    )]
    pub after: Option<DateTime<Utc>>,
    pub before: Option<DateTime<Utc>>,
    #[param(ignore)]
    pub operation_date: bool,
    #[param(operator = ">=", field = "amount")]
    pub greater_than: Option<Decimal>,
    #[param(operator = "<", field = "amount")]
    pub less_than: Option<Decimal>,
    pub direction: Option<Direction>,
    pub mode: Option<Mode>,
    pub merchant_id: Option<Option<Id>>,
    pub category_id: Option<Option<Id>>,
    #[param(operator = "LIKE")]
    pub details: Option<String>,
    #[param(limit)]
    pub count: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query() -> anyhow::Result<()> {
        let query = QueryRecord {
            account_id: Some(0.into()),
            after: Some(Utc::now()),
            operation_date: true,
            greater_than: Some(oxydized_money::Decimal::from(10).into()),
            less_than: Some(oxydized_money::Decimal::from(100).into()),
            count: Some(5),
            ..Default::default()
        };

        println!("{:#?}", query);

        Ok(())
    }
}
