use crate::{essentials::*, record::Direction, schema::records};

use std::ops::Range;

use chrono::NaiveDate;
use diesel::prelude::*;

#[derive(derive_more::From, derive_more::Deref)]
pub struct CategoriesStats(pub Vec<CategoryStats>);

impl CategoriesStats {
    pub fn from_date_range_and_currency(
        conn: &mut Conn,
        range: Range<NaiveDate>,
        currency: Currency,
    ) -> Result<Self> {
        let stats = records::table
            .filter(records::operation_date.ge(range.start))
            .filter(records::operation_date.lt(range.end))
            .filter(records::currency.eq(db::Currency::from(currency)))
            .group_by((records::currency, records::direction, records::category_id))
            .select(CategoryStats::as_select())
            .load::<CategoryStats>(conn)?;

        Ok(stats.into())
    }
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct CategoryStats {
    #[diesel(select_expression = records::category_id)]
    pub category_id: Option<i64>,
    #[diesel(select_expression = records::direction)]
    pub direction: Direction,
    #[diesel(
        select_expression = db::total(records::amount),
        deserialize_as = db::Decimal
    )]
    pub amount: Decimal,
    #[diesel(
        select_expression = records::currency,
        deserialize_as = db::Currency
    )]
    pub currency: Currency,
}

impl CategoryStats {
    pub fn amount(&self) -> Amount {
        Amount(self.amount, self.currency)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::account::NewAccount;
    use crate::record::NewRecord;
    use crate::test::prelude::{assert_eq, Result, *};

    #[test]
    fn from_date_range_and_currency() -> Result<()> {
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
            }
            .save(conn)?;
        }

        let stats = CategoriesStats::from_date_range_and_currency(conn, start..end, Currency::EUR)?;

        assert_eq!(
            Decimal::new(942, 2),
            stats.iter().fold(Decimal::ZERO, |acc, e| acc + e.amount)
        );

        let cat1_stats = stats
            .iter()
            .find(|e| e.category_id == Some(cat1.id))
            .unwrap();
        assert_eq!(Decimal::new(314, 2), cat1_stats.amount);

        let cat2_stats = stats
            .iter()
            .find(|e| e.category_id == Some(cat2.id))
            .unwrap();
        assert_eq!(Decimal::new(628, 2), cat2_stats.amount);

        let cat3_stats = stats.iter().find(|e| e.category_id == Some(cat3.id));
        assert!(cat3_stats.is_none());

        Ok(())
    }

    #[test]
    fn without_category() -> Result<()> {
        let conn = &mut test::db()?;
        let account = &test::account(conn, "account")?;

        let start = NaiveDate::from_ymd_opt(2024, 2, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();

        NewRecord {
            amount: Decimal::new(420, 2),
            operation_date: start,
            ..NewRecord::new(account)
        }
        .save(conn)?;

        let stats = CategoriesStats::from_date_range_and_currency(conn, start..end, Currency::EUR)?;
        assert_eq!(
            Decimal::new(420, 2),
            stats.iter().fold(Decimal::ZERO, |acc, e| acc + e.amount)
        );

        let nocat_stats = stats.iter().find(|e| e.category_id.is_none()).unwrap();
        assert_eq!(Decimal::new(420, 2), nocat_stats.amount);

        Ok(())
    }

    #[test]
    fn multiple_currencies() -> Result<()> {
        let conn = &mut test::db()?;
        let euro = &NewAccount {
            currency: Currency::EUR,
            ..NewAccount::new("euro")
        }
        .save(conn)?;
        let dollar = &NewAccount {
            currency: Currency::USD,
            ..NewAccount::new("dollar")
        }
        .save(conn)?;

        let start = NaiveDate::from_ymd_opt(2024, 2, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 3, 1).unwrap();

        NewRecord {
            amount: Decimal::new(420, 2),
            operation_date: start,
            ..NewRecord::new(euro)
        }
        .save(conn)?;
        NewRecord {
            amount: Decimal::new(210, 2),
            operation_date: start,
            ..NewRecord::new(dollar)
        }
        .save(conn)?;

        let stats = CategoriesStats::from_date_range_and_currency(conn, start..end, Currency::EUR)?;
        assert!(stats.iter().all(|e| e.currency == Currency::EUR));
        assert_eq!(
            Decimal::new(420, 2),
            stats.iter().fold(Decimal::ZERO, |acc, e| acc + e.amount)
        );

        let stats = CategoriesStats::from_date_range_and_currency(conn, start..end, Currency::USD)?;
        assert!(stats.iter().all(|e| e.currency == Currency::USD));
        assert_eq!(
            Decimal::new(210, 2),
            stats.iter().fold(Decimal::ZERO, |acc, e| acc + e.amount)
        );

        Ok(())
    }
}
