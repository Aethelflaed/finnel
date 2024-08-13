use crate::{
    essentials::*,
    schema::{monthly_stats, monthly_stats_category},
    Amount, Currency, Decimal,
};
use diesel::prelude::*;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = monthly_stats)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct MonthlyStats {
    pub year: i32,
    pub month: i32,
    #[diesel(deserialize_as = crate::db::Decimal)]
    pub amount: Decimal,
    #[diesel(deserialize_as = crate::db::Currency)]
    pub currency: Currency,
}

impl MonthlyStats {
    pub fn amount(&self) -> Amount {
        Amount(self.amount, self.currency)
    }
}

#[derive(Debug, Queryable, Selectable, Identifiable)]
#[diesel(table_name = monthly_stats_category)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct MonthlyStatsCategory {
    pub id: i64,
    pub year: i32,
    pub month: i32,
    #[diesel(deserialize_as = crate::db::Decimal)]
    pub amount: Decimal,
    #[diesel(deserialize_as = crate::db::Currency)]
    pub currency: Currency,
    pub category_id: i64,
}

impl MonthlyStatsCategory {
    pub fn amount(&self) -> Amount {
        Amount(self.amount, self.currency)
    }
}
