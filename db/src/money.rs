use oxydized_money::CurrencyError;
use rusqlite::{
    types::{
        FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, Value,
        ValueRef,
    },
    Error, Result,
};

#[derive(Copy, Clone, Debug, derive_more::From, derive_more::Into)]
pub struct Decimal(oxydized_money::Decimal);

#[derive(Copy, Clone, Debug, derive_more::From, derive_more::Into)]
pub struct Currency(oxydized_money::Currency);

impl FromSql for Decimal {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        Ok(oxydized_money::Decimal::new(value.as_i64()?, 3).into())
    }
}

impl ToSql for Decimal {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
        let mut value = self.0;
        value.rescale(3);
        match TryInto::<i64>::try_into(value.mantissa()) {
            Ok(value) => Ok(ToSqlOutput::Owned(Value::Integer(value))),
            Err(e) => Err(Error::ToSqlConversionFailure(Box::new(e))),
        }
    }
}

impl FromSql for Currency {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        match oxydized_money::Currency::from_code(value.as_str()?) {
            Some(cur) => Ok(Currency(cur)),
            None => Err(FromSqlError::Other(Box::new(CurrencyError::Unknown))),
        }
    }
}

impl ToSql for Currency {
    fn to_sql(&self) -> Result<ToSqlOutput<'_>> {
        self.0.code().to_sql()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    use crate::{DatabaseTrait, Id, Result, SimpleDatabase};

    use oxydized_money::Amount;
    use rusqlite::params;

    #[test]
    fn read_and_write() -> Result<()> {
        let db = SimpleDatabase::memory()?;

        db.execute(
            "CREATE TABLE money_test (
                amount TEXT NOT NULL,
                currency TEXT NOT NULL
            );",
            (),
        )?;

        let amount = Amount(
            oxydized_money::Decimal::from_str_exact("3.14").unwrap(),
            oxydized_money::Currency::EUR,
        );
        let query = "INSERT INTO money_test(amount, currency)
            VALUES(?, ?) RETURNING rowid;";
        let mut id = Option::<Id>::None;
        db.prepare(query)?.query_row(
            params![Decimal(amount.0), Currency(amount.1)],
            |row| {
                id = Some(row.get(0)?);
                Ok(())
            },
        )?;

        let query = "SELECT amount, currency
            FROM money_test
            WHERE rowid = ? LIMIT 1";

        db.prepare(query)?.query_row([id.unwrap()], |row| {
            assert_eq!(amount.0, row.get::<usize, Decimal>(0)?.into());
            assert_eq!(amount.1, row.get::<usize, Currency>(1)?.into());
            Ok(())
        })?;

        Ok(())
    }
}
