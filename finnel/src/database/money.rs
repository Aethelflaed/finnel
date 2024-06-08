use oxydized_money::{Amount, Currency, CurrencyError, Decimal};
use rusqlite::types::{FromSql, FromSqlResult, ToSql, ToSqlOutput, ValueRef};

use crate::database::Result;

#[derive(Copy, Clone, Debug, derive_more::From, derive_more::Into)]
pub struct Money(Amount);

impl Default for Money {
    fn default() -> Self {
        Self(Amount(Decimal::new(0, 0), Currency::EUR))
    }
}

impl Money {
    fn serialize(&self) -> Vec<u8> {
        let mut vec = Vec::<u8>::with_capacity(18);
        vec.extend_from_slice(&self.0 .0.serialize());
        vec.extend_from_slice(&self.0 .1.numeric().to_be_bytes());
        vec
    }

    fn deserialize(vec: &[u8]) -> Result<Self> {
        Ok(Self(Amount(
            Decimal::deserialize(vec[0..16].try_into()?),
            Currency::from_numeric(u16::from_be_bytes(vec[16..18].try_into()?))
                .ok_or(CurrencyError::Unknown)?,
        )))
    }
}

impl FromSql for Money {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        Self::deserialize(value.as_bytes()?)
            .map_err(|e| rusqlite::types::FromSqlError::Other(Box::new(e)))
    }
}

impl ToSql for Money {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Owned(rusqlite::types::Value::Blob(
            self.serialize(),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    use crate::database::{Database, Id};

    #[test]
    fn read_and_write() -> Result<()> {
        let db = Database::memory().unwrap();
        db.connection.execute(
            "CREATE TABLE money_test (
                amount BLOB NOT NULL
            );",
            (),
        )?;

        let amount =
            Amount(Decimal::from_str_exact("3.14").unwrap(), Currency::EUR);
        let query = "INSERT INTO money_test(amount) VALUES(?) RETURNING rowid;";
        let mut stmt = db.connection.prepare(query)?;
        let mut id = Id::from(0);
        stmt.query_row([Money(amount)], |row| {
            id = row.get(0)?;
            Ok(())
        })?;

        let query = "SELECT amount FROM money_test WHERE rowid = ? LIMIT 1";
        stmt = db.connection.prepare(query)?;

        stmt.query_row([id], |row| {
            assert_eq!(amount, Amount::from(row.get::<usize, Money>(0)?));
            Ok(())
        })?;

        Ok(())
    }
}
