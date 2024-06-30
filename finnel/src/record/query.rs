use chrono::{offset::Utc, DateTime};

use crate::record::Record;
use crate::transaction::{Direction, Mode};
use db::{Decimal, Id, Query};

use rusqlite::ToSql;

use derive::Query;

#[derive(Debug, Default, Query)]
#[query(entity = Record, table = "records")]
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

        println!("{}", query.query());

        for (key, value) in query.params() {
            println!("{} => {:?}", key, value.to_sql()?);
        }

        Ok(())
    }
}
