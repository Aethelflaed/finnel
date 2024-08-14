use crate::{
    essentials::*,
    schema::{categories, records},
};

use std::ops::Range;

use chrono::NaiveDate;
use diesel::prelude::*;

#[derive(Debug)]
pub struct CategoriesStats {
    pub stats: Vec<CategoryStats>,
    pub amount: Decimal,
}

impl CategoriesStats {
    pub fn from_date_range(conn: &mut Conn, range: Range<NaiveDate>) -> Result<Self> {
        let stats = categories::table
            .inner_join(records::table)
            .filter(records::operation_date.ge(range.start))
            .filter(records::operation_date.lt(range.end))
            .group_by(categories::id)
            .select(CategoryStats::as_select())
            .load::<CategoryStats>(conn)?;

        Ok(stats.into())
    }
}

impl From<Vec<CategoryStats>> for CategoriesStats {
    fn from(vec: Vec<CategoryStats>) -> Self {
        let total = vec.iter().fold(Decimal::new(0, 0), |acc, e| acc + e.amount);

        Self {
            stats: vec,
            amount: total,
        }
    }
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct CategoryStats {
    #[diesel(select_expression = categories::id)]
    pub category_id: i64,
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
        let cat1 = &test::category(conn, "cat1")?;
        let cat2 = &test::category(conn, "cat2")?;
        let cat3 = &test::category(conn, "cat3")?;

        let acc1 = &test::account(conn, "acc1")?;
        let acc2 = &test::account(conn, "acc2")?;

        let before = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        let start = NaiveDate::from_ymd_opt(2024, 2, 1).unwrap();
        let during1 = NaiveDate::from_ymd_opt(2024, 2, 5).unwrap();
        let during2 = NaiveDate::from_ymd_opt(2024, 2, 29).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();
        let after = NaiveDate::from_ymd_opt(2024, 3, 2).unwrap();

        let dates = [before, start, during1, during2, end, after];
        let categories = [&cat1, &cat1, &cat2, &cat2, &cat1, &cat2];
        let accounts = [&acc2, &acc1, &acc2, &acc1, &acc2, &acc1];

        for (pos, date) in dates.iter().enumerate() {
            NewRecord {
                amount: Decimal::new(314, 2),
                operation_date: *date,
                category: Some(categories[pos]),
                ..NewRecord::new(accounts[pos])
            }.save(conn)?;
        }

        let stats = CategoriesStats::from_date_range(conn, start..end)?;

        assert_eq!(Decimal::new(942, 2), stats.amount);
        
        let cat1_stats = stats.stats.iter().find(|e| e.category_id == cat1.id).unwrap();
        assert_eq!(Decimal::new(314, 2), cat1_stats.amount);

        let cat2_stats = stats.stats.iter().find(|e| e.category_id == cat2.id).unwrap();
        assert_eq!(Decimal::new(628, 2), cat2_stats.amount);

        let cat3_stats = stats.stats.iter().find(|e| e.category_id == cat3.id);
        assert!(cat3_stats.is_none());

        Ok(())
    }
}
