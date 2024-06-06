use std::str::FromStr;

use chrono::{offset::Utc, DateTime};

use oxydized_money::Amount;

use crate::database::{
    Amount as DbAmount, Database, Date, Entity, Error, Readable, Result,
    Upgrade,
};

pub use crate::database::Id;
use crate::{account, category, database, merchant, transaction};

pub struct Record {
    id: Option<Id>,
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
            id: Some(Id::from(statement.read::<i64, _>("id")?)),
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

impl Entity for Record {
    fn id(&self) -> Option<Id> {
        self.id
    }

    fn find(db: &Database, id: Id) -> Result<Self> {
        let query = "SELECT * FROM records WHERE id = ? LIMIT 1;";
        let mut statement = db.connection.prepare(query)?;
        statement.bind((1, id))?;

        if let Ok(sqlite::State::Row) = statement.next() {
            statement.try_into()
        } else {
            Err(Error::NotFound)
        }
    }

    fn save(&mut self, db: &Database) -> Result<()> {
        if let Some(id) = self.id {
            let query = "UPDATE records SET
                    value_date = :value_date,
                    category = :category,
                    merchant = :merchant
                WHERE id = :id";
            let mut statement = db.connection.prepare(query)?;
            statement.bind((":id", id))?;

            statement
                .bind((":value_date", self.value_date.to_string().as_str()))?;
            statement.bind((":category", self.category))?;
            statement.bind((":merchant", self.merchant))?;

            if let Ok(sqlite::State::Done) = statement.next() {
                Ok(())
            } else {
                Err(Error::NotFound)
            }
        } else {
            let query = "
                INSERT INTO records (
                    account,
                    amount_val, amount_cur,
                    operation_date, value_date,
                    transaction_type, transaction_details,
                    category,
                    merchant
                ) VALUES (
                    :account,
                    :amount_val, :amount_cur,
                    :operation_date, :value_date,
                    :transaction_type, :transaction_details,
                    :category,
                    :merchant
                )
                RETURNING id;";
            let mut statement = db.connection.prepare(query)?;
            statement.bind((":account", self.account))?;

            let db_amount = DbAmount::from(self.amount);
            statement.bind((":amount_val", db_amount.val().as_str()))?;
            statement.bind((":amount_cur", db_amount.cur()))?;

            statement.bind((
                ":operation_date",
                self.operation_date.to_string().as_str(),
            ))?;
            statement
                .bind((":value_date", self.value_date.to_string().as_str()))?;
            statement.bind((
                ":transaction_type",
                self.transaction_type.to_string().as_str(),
            ))?;
            statement.bind((
                ":transaction_details",
                self.transaction_details.as_str(),
            ))?;
            statement.bind((":category", self.category))?;
            statement.bind((":merchant", self.merchant))?;

            if let Ok(sqlite::State::Row) = statement.next() {
                self.id = Some(Id::try_read("id", &statement)?);
                Ok(())
            } else {
                Err(Error::NotFound)
            }
        }
    }
}

impl Upgrade for Record {
    fn upgrade_from(db: &Database, _version: &semver::Version) -> Result<()> {
        db.connection
            .execute(
                "
                CREATE TABLE IF NOT EXISTS records (
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
        Record::setup(&db).unwrap();
    }
}
