use std::str::FromStr;

use chrono::{offset::Utc, DateTime};

use oxydized_money::Amount;

use crate::database::{Amount as DbAmount, Database, Date, Error, Result};

pub use crate::database::Id;
use crate::{account, category, database, merchant, transaction};

pub struct Record {
    id: Id,
    account: account::Id,
    amount: Amount,
    operation_date: DateTime<Utc>,
    value_date: DateTime<Utc>,
    transaction_type: transaction::Type,
    transaction_details: String,
    category: category::Id,
    merchant: merchant::Id,
}

impl TryFrom<sqlite::Statement<'_>> for Record {
    type Error = Error;

    fn try_from(statement: sqlite::Statement) -> Result<Self> {
        Ok(Record {
            id: Id::from(statement.read::<i64, _>("id")?),
            account: account::Id::from(statement.read::<i64, _>("account")?),
            amount: DbAmount::try_read("amount", &statement)?.into(),
            operation_date: Date::try_read("operation_date", &statement)?
                .into(),
            value_date: Date::try_read("value_date", &statement)?.into(),
            transaction_type: transaction::Type::from_str(
                &statement.read::<String, _>("transaction_type")?,
            )?,
            transaction_details: statement
                .read::<String, _>("transaction_details")?,
            category: category::Id::from(statement.read::<i64, _>("category")?),
            merchant: merchant::Id::from(statement.read::<i64, _>("merchant")?),
        })
    }
}

pub trait RecordStorage {
    fn find(&self, id: Id) -> Result<Record>;
    fn setup(&self) -> Result<()>;
}

impl RecordStorage for Database {
    fn find(&self, id: Id) -> Result<Record> {
        let query = "SELECT * FROM records WHERE id = ? LIMIT 1;";
        let mut statement = self.connection.prepare(query).unwrap();
        statement.bind((1, id)).unwrap();

        if let Ok(sqlite::State::Row) = statement.next() {
            statement.try_into()
        } else {
            Err(Error::NotFound)
        }
    }

    fn setup(&self) -> Result<()> {
        self.connection
            .execute(
                "
                CREATE TABLE records (
                    id INTEGER NOT NULL PRIMARY KEY,
                    account INTEGER NOT NULL,
                    amount_val TEXT NOT NULL,
                    amount_cur TEXT NOT NULL,
                    operation_date TEXT NOT NULL,
                    value_date TEXT NOT NULL,
                    transaction_type TEXT,
                    transaction_details TEXT,
                    category INTEGER,
                    merchant INTEGER
                );
            ",
            )
            .map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn setup() {
        let db = Database::memory().unwrap();
        RecordStorage::setup(&db).unwrap();
    }
}
