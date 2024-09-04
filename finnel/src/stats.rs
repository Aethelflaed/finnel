use crate::{
    date,
    essentials::*,
    record::Direction,
    schema::{monthly_category_stats, monthly_stats},
};

use diesel::prelude::*;

mod categories;
pub use categories::{CategoriesStats, CategoryStats};

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = monthly_stats)]
#[diesel(primary_key(year, month, currency))]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct MonthlyStats {
    pub year: i32,
    pub month: i32,
    #[diesel(deserialize_as = db::Decimal)]
    pub debit_amount: Decimal,
    #[diesel(deserialize_as = db::Decimal)]
    pub credit_amount: Decimal,
    #[diesel(deserialize_as = db::Currency, serialize_as = db::Currency)]
    pub currency: Currency,
}

// Required by Identifiable, but doesn't have its own derive macro
impl diesel::associations::HasTable for MonthlyStats {
    type Table = monthly_stats::table;

    fn table() -> Self::Table {
        monthly_stats::table
    }
}

// derive(Identifiable) does not honors `serialize_as` and would generate a type that's not
// an expression, so we need to derive it manually
// cf https://github.com/diesel-rs/diesel/pull/4168
impl Identifiable for &MonthlyStats {
    type Id = (i32, i32, db::Currency);

    fn id(self) -> Self::Id {
        (self.year, self.month, db::Currency::from(self.currency))
    }
}

impl MonthlyStats {
    pub fn debit_amount(&self) -> Amount {
        Amount(self.debit_amount, self.currency)
    }

    pub fn credit_amount(&self) -> Amount {
        Amount(self.credit_amount, self.currency)
    }

    pub fn find_or_create(
        conn: &mut Conn,
        year: i32,
        month: i32,
        currency: Currency,
    ) -> Result<Self> {
        // Check if it's possible to build a date range with the given year/month first
        date::Month::calendar(year, month).as_date_range()?;

        if let Some(instance) = monthly_stats::table
            .filter(monthly_stats::year.eq(year))
            .filter(monthly_stats::month.eq(month))
            .filter(monthly_stats::currency.eq(db::Currency::from(currency)))
            .select(MonthlyStats::as_select())
            .first(conn)
            .optional()?
        {
            Ok(instance)
        } else {
            Self::create(conn, year, month, currency)
        }
    }

    pub fn create(conn: &mut Conn, year: i32, month: i32, currency: Currency) -> Result<Self> {
        // Check if it's possible to build a date range with the given year/month first
        date::Month::calendar(year, month).as_date_range()?;

        let mut monthly_stats = diesel::insert_into(monthly_stats::table)
            .values((
                monthly_stats::year.eq(year),
                monthly_stats::month.eq(month),
                monthly_stats::debit_amount.eq(db::Decimal::from(Decimal::ZERO)),
                monthly_stats::credit_amount.eq(db::Decimal::from(Decimal::ZERO)),
                monthly_stats::currency.eq(db::Currency::from(currency)),
            ))
            .returning(MonthlyStats::as_select())
            .get_result(conn)?;

        monthly_stats.rebuild(conn)?;

        Ok(monthly_stats)
    }

    pub fn rebuild(&mut self, conn: &mut Conn) -> Result<()> {
        self.delete_category_stats(conn)?;

        self.debit_amount = Decimal::new(0, 0);
        self.credit_amount = Decimal::new(0, 0);

        let stats = CategoriesStats::from_date_range_and_currency(
            conn,
            date::Month::calendar(self.year, self.month).as_date_range()?,
            self.currency,
        )?;

        let monthly_category_stats = stats
            .0
            .into_iter()
            .map(|category_stats| {
                if category_stats.direction.is_debit() {
                    self.debit_amount += category_stats.amount;
                } else {
                    self.credit_amount += category_stats.amount;
                }

                MonthlyCategoryStats {
                    id: -1,
                    year: self.year,
                    month: self.month,
                    amount: category_stats.amount,
                    currency: category_stats.currency,
                    category_id: category_stats.category_id,
                    direction: category_stats.direction,
                }
            })
            .collect::<Vec<MonthlyCategoryStats>>();

        if !monthly_category_stats.is_empty() {
            diesel::insert_into(monthly_category_stats::table)
                .values(monthly_category_stats)
                .execute(conn)?;
        }

        diesel::update(&*self)
            .set((
                monthly_stats::debit_amount.eq(db::Decimal::from(self.debit_amount)),
                monthly_stats::credit_amount.eq(db::Decimal::from(self.credit_amount)),
            ))
            .execute(conn)?;

        Ok(())
    }

    fn delete_category_stats(&self, conn: &mut Conn) -> Result<()> {
        diesel::delete(monthly_category_stats::table)
            .filter(monthly_category_stats::year.eq(self.year))
            .filter(monthly_category_stats::month.eq(self.month))
            .filter(monthly_category_stats::currency.eq(db::Currency::from(self.currency)))
            .execute(conn)?;
        Ok(())
    }
}

#[derive(Debug, Queryable, Selectable, Identifiable, Insertable)]
#[diesel(table_name = monthly_category_stats)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct MonthlyCategoryStats {
    #[diesel(skip_insertion)]
    pub id: i64,
    pub year: i32,
    pub month: i32,
    #[diesel(deserialize_as = db::Decimal, serialize_as = db::Decimal)]
    pub amount: Decimal,
    #[diesel(deserialize_as = db::Currency, serialize_as = db::Currency)]
    pub currency: Currency,
    pub category_id: Option<i64>,
    pub direction: Direction,
}

impl MonthlyCategoryStats {
    pub fn amount(&self) -> Amount {
        Amount(self.amount, self.currency)
    }
}

pub(crate) fn clear_category_id(conn: &mut Conn, id: i64) -> Result<()> {
    diesel::delete(monthly_category_stats::table)
        .filter(monthly_category_stats::category_id.eq(Some(id)))
        .execute(conn)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::NewRecord;
    use crate::test::prelude::{assert_eq, Result, *};
    use chrono::NaiveDate;
    use diesel::dsl::count_star;

    #[test]
    fn create_empty_then_find_or_create() -> Result<()> {
        let conn = &mut test::db()?;

        assert_eq!(0i64, monthly_stats::table.select(count_star()).first(conn)?);

        let stats = MonthlyStats::create(conn, 2024, 8, Currency::EUR)?;
        assert_eq!(Decimal::ZERO, stats.debit_amount);

        MonthlyStats::find_or_create(conn, 2024, 8, Currency::EUR)?;

        assert_eq!(1i64, monthly_stats::table.select(count_star()).first(conn)?);

        Ok(())
    }

    #[test]
    fn create() -> Result<()> {
        let conn = &mut test::db()?;
        let account = &test::account!(conn, "Cash");
        NewRecord {
            amount: Decimal::new(314, 2),
            operation_date: NaiveDate::from_ymd_opt(2024, 8, 1).unwrap(),
            ..NewRecord::new(account)
        }
        .save(conn)?;

        let stats = MonthlyStats::create(conn, 2024, 8, Currency::EUR)?;
        assert_eq!(Decimal::new(314, 2), stats.debit_amount);

        Ok(())
    }

    #[test]
    fn rebuild_deletes_existing_category_stats() -> Result<()> {
        let conn = &mut test::db()?;
        let mut stats = MonthlyStats::create(conn, 2024, 8, Currency::EUR)?;

        diesel::insert_into(monthly_category_stats::table)
            .values([
                MonthlyCategoryStats {
                    id: 0,
                    year: 2024,
                    month: 8,
                    amount: Decimal::ZERO,
                    currency: Currency::EUR,
                    category_id: None,
                    direction: Direction::Debit,
                },
                MonthlyCategoryStats {
                    id: 0,
                    year: 2024,
                    month: 8,
                    amount: Decimal::ZERO,
                    currency: Currency::USD,
                    category_id: None,
                    direction: Direction::Debit,
                },
                MonthlyCategoryStats {
                    id: 0,
                    year: 2024,
                    month: 7,
                    amount: Decimal::ZERO,
                    currency: Currency::EUR,
                    category_id: None,
                    direction: Direction::Debit,
                },
            ])
            .execute(conn)?;

        assert_eq!(
            3i64,
            monthly_category_stats::table
                .select(count_star())
                .first(conn)?
        );
        stats.rebuild(conn)?;
        assert_eq!(
            2i64,
            monthly_category_stats::table
                .select(count_star())
                .first(conn)?
        );

        Ok(())
    }

    #[test]
    fn rebuild() -> Result<()> {
        let conn = &mut test::db()?;
        let mut stats = MonthlyStats::create(conn, 2024, 8, Currency::EUR)?;

        let date = NaiveDate::from_ymd_opt(2024, 8, 1).unwrap();
        let account = &test::account!(conn, "account");

        let cat1 = &test::category!(conn, "cat1");
        let cat2 = &test::category!(conn, "cat2");
        let cat3 = &test::category!(conn, "cat3");

        let categories = [Some(&cat1), Some(&cat1), Some(&cat2), None];
        for category in categories {
            NewRecord {
                amount: Decimal::new(314, 2),
                operation_date: date,
                category: category.copied(),
                direction: Direction::Debit,
                ..NewRecord::new(account)
            }
            .save(conn)?;

            NewRecord {
                amount: Decimal::new(420, 2),
                operation_date: date,
                category: category.copied(),
                direction: Direction::Credit,
                ..NewRecord::new(account)
            }
            .save(conn)?;
        }

        stats.rebuild(conn)?;
        assert_eq!(Decimal::new(1680, 2), stats.credit_amount);
        assert_eq!(Decimal::new(1256, 2), stats.debit_amount);

        let category_stats = monthly_category_stats::table
            .select(MonthlyCategoryStats::as_select())
            .load::<MonthlyCategoryStats>(conn)?;
        assert_eq!(6, category_stats.len());

        assert!(category_stats
            .iter()
            .all(|c| c.category_id != Some(cat3.id)));
        assert!(category_stats.iter().any(|c| c.category_id.is_none()));

        Ok(())
    }

    #[test]
    fn delete_category() -> Result<()> {
        let conn = &mut test::db()?;

        let mut stats = MonthlyStats::create(conn, 2024, 8, Currency::EUR)?;
        let mut cat1 = test::category!(conn, "cat1");

        let date = NaiveDate::from_ymd_opt(2024, 8, 1).unwrap();
        let account = &test::account!(conn, "account");

        NewRecord {
            amount: Decimal::new(314, 2),
            operation_date: date,
            category: Some(&cat1),
            direction: Direction::Debit,
            ..NewRecord::new(account)
        }
        .save(conn)?;
        stats.rebuild(conn)?;

        assert_eq!(
            1i64,
            monthly_category_stats::table.select(count_star()).first(conn)?
        );

        cat1.delete(conn)?;

        assert_eq!(
            0i64,
            monthly_category_stats::table.select(count_star()).first(conn)?
        );

        Ok(())
    }
}
