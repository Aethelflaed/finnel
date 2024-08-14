use crate::{essentials::*, schema::records};

use std::ops::Range;

use chrono::NaiveDate;
use diesel::prelude::*;

pub struct MerchantsStats {
    pub stats: Vec<MerchantStats>,
    pub total: Option<AmountResult>,
}

impl MerchantsStats {
    pub fn from_date_range(conn: &mut Conn, range: Range<NaiveDate>) -> Result<Self> {
        let stats = records::table
            .filter(records::operation_date.ge(range.start))
            .filter(records::operation_date.lt(range.end))
            .group_by((records::currency, records::merchant_id))
            .select(MerchantStats::as_select())
            .load::<MerchantStats>(conn)?;

        Ok(stats.into())
    }

    pub fn total(&self) -> Result<Option<Amount>> {
        Ok(self.total.map(|t| t.into_inner()).transpose()?)
    }
}

impl From<Vec<MerchantStats>> for MerchantsStats {
    fn from(vec: Vec<MerchantStats>) -> Self {
        let total = vec
            .iter()
            .map(|e| AmountResult::from(e.amount()))
            .reduce(|acc, e| acc + e);

        Self { stats: vec, total }
    }
}

#[derive(Queryable, Selectable)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct MerchantStats {
    #[diesel(select_expression = records::merchant_id)]
    pub merchant_id: Option<i64>,
    #[diesel(
        select_expression = db::total(records::amount),
        deserialize_as = db::Decimal
    )]
    pub amount: Decimal,
    #[diesel(
        select_expression = records::currency,
        deserialize_as = crate::db::Currency
    )]
    pub currency: Currency,
}

impl MerchantStats {
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
            }
            .save(conn)?;
        }

        let stats = MerchantsStats::from_date_range(conn, start..end)?;

        assert_eq!(
            Some(Amount(Decimal::new(942, 2), Currency::EUR)),
            stats.total()?
        );

        let mer1_stats = stats
            .stats
            .iter()
            .find(|e| e.merchant_id == Some(mer1.id))
            .unwrap();
        assert_eq!(Decimal::new(314, 2), mer1_stats.amount);

        let mer2_stats = stats
            .stats
            .iter()
            .find(|e| e.merchant_id == Some(mer2.id))
            .unwrap();
        assert_eq!(Decimal::new(628, 2), mer2_stats.amount);

        let mer3_stats = stats.stats.iter().find(|e| e.merchant_id == Some(mer3.id));
        assert!(mer3_stats.is_none());

        Ok(())
    }

    #[test]
    fn without_merchant() -> Result<()> {
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

        let stats = MerchantsStats::from_date_range(conn, start..end)?;
        assert_eq!(
            Some(Amount(Decimal::new(420, 2), Currency::EUR)),
            stats.total()?
        );

        let stats = stats
            .stats
            .iter()
            .find(|e| e.merchant_id.is_none())
            .unwrap();
        assert_eq!(Decimal::new(420, 2), stats.amount);

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
            amount: Decimal::new(420, 2),
            operation_date: start,
            ..NewRecord::new(dollar)
        }
        .save(conn)?;

        let stats = MerchantsStats::from_date_range(conn, start..end)?;
        assert!(stats.total.unwrap().is_mismatch());

        let eur_stats = stats
            .stats
            .iter()
            .find(|e| e.currency == Currency::EUR)
            .unwrap();
        assert_eq!(Decimal::new(420, 2), eur_stats.amount);
        let usd_stats = stats
            .stats
            .iter()
            .find(|e| e.currency == Currency::USD)
            .unwrap();
        assert_eq!(Decimal::new(420, 2), usd_stats.amount);

        Ok(())
    }
}
