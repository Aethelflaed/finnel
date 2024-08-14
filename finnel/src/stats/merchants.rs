use crate::{
    essentials::*,
    schema::{merchants, records},
};

use std::ops::Range;

use chrono::NaiveDate;
use diesel::prelude::*;

pub struct MerchantsStats {
    pub stats: Vec<MerchantStats>,
    pub amount: Decimal,
}

impl MerchantsStats {
    pub fn from_date_range(conn: &mut Conn, range: Range<NaiveDate>) -> Result<Self> {
        let stats = merchants::table
            .inner_join(records::table)
            .filter(records::operation_date.ge(range.start))
            .filter(records::operation_date.lt(range.end))
            .group_by(merchants::id)
            .select(MerchantStats::as_select())
            .load::<MerchantStats>(conn)?;

        Ok(stats.into())
    }
}

impl From<Vec<MerchantStats>> for MerchantsStats {
    fn from(vec: Vec<MerchantStats>) -> Self {
        let total = vec.iter().fold(Decimal::new(0, 0), |acc, e| acc + e.amount);

        Self {
            stats: vec,
            amount: total,
        }
    }
}

#[derive(Queryable, Selectable)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct MerchantStats {
    #[diesel(select_expression = merchants::id)]
    pub merchant_id: i64,
    #[diesel(
        select_expression = db::total(records::amount),
        deserialize_as = db::Decimal
    )]
    pub amount: Decimal,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::prelude::{*, assert_eq, Result};
    use crate::record::NewRecord;

    #[test]
    fn from_date_range() -> Result<()> {
        let conn = &mut test::db()?;
        let mer1 = &test::merchant(conn, "mer1")?;
        let mer2 = &test::merchant(conn, "mer2")?;
        let mer3 = &test::merchant(conn, "mer3")?;

        let acc1 = &test::account(conn, "acc1")?;
        let acc2 = &test::account(conn, "acc2")?;

        let before = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        let start = NaiveDate::from_ymd_opt(2024, 2, 1).unwrap();
        let during1 = NaiveDate::from_ymd_opt(2024, 2, 5).unwrap();
        let during2 = NaiveDate::from_ymd_opt(2024, 2, 29).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let after = NaiveDate::from_ymd_opt(2024, 3, 2).unwrap();

        let dates = [before, start, during1, during2, end, after];
        let merchants = [&mer1, &mer1, &mer2, &mer2, &mer1, &mer2];
        let accounts = [&acc2, &acc1, &acc2, &acc1, &acc2, &acc1];

        for (pos, date) in dates.iter().enumerate() {
            NewRecord {
                amount: Decimal::new(314, 2),
                operation_date: *date,
                merchant: Some(merchants[pos]),
                ..NewRecord::new(accounts[pos])
            }.save(conn)?;
        }

        let stats = MerchantsStats::from_date_range(conn, start..end)?;

        assert_eq!(Decimal::new(942, 2), stats.amount);
        
        let mer1_stats = stats.stats.iter().find(|e| e.merchant_id == mer1.id).unwrap();
        assert_eq!(Decimal::new(314, 2), mer1_stats.amount);

        let mer2_stats = stats.stats.iter().find(|e| e.merchant_id == mer2.id).unwrap();
        assert_eq!(Decimal::new(628, 2), mer2_stats.amount);

        let mer3_stats = stats.stats.iter().find(|e| e.merchant_id == mer3.id);
        assert!(mer3_stats.is_none());

        Ok(())
    }
}
