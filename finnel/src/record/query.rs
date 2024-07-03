use chrono::{offset::Utc, DateTime};

use crate::transaction::{Direction, Mode};
use crate::{Category, Merchant, Record};
use db::{Decimal, Id, Query, Row};

use rusqlite::ToSql;

use derive::{Query, QueryDebug};

pub struct FullRecord {
    pub record: Record,
    pub merchant: Merchant,
    pub category: Category,
}

impl TryFrom<&Row<'_>> for FullRecord {
    type Error = rusqlite::Error;

    fn try_from(row: &Row) -> rusqlite::Result<Self> {
        Ok(FullRecord {
            record: row.with_prefix("records_", |row| Record::try_from(row))?,
            merchant: row
                .with_prefix("merchants_", |row| Merchant::try_from(row))?,
            category: row
                .with_prefix("categories_", |row| Category::try_from(row))?,
        })
    }
}

#[derive(Default, Query, QueryDebug)]
#[query(result = FullRecord, entity = Record, alias = "record")]
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
    #[param(join(inner, entity = Merchant, field = "id"))]
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
