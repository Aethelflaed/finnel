use chrono::{offset::Utc, DateTime};

use oxydized_money::Amount;

use crate::database::{
    Connection, Database, Entity, Error, Id, Money, Result, Upgrade,
};

use crate::transaction;

pub struct Record {
    id: Option<Id>,
    account: Id,
    amount: Amount,
    operation_date: DateTime<Utc>,
    value_date: DateTime<Utc>,
    transaction_type: Option<transaction::Type>,
    transaction_details: String,
    category: Option<Id>,
    merchant: Option<Id>,
}

impl Record {
    pub fn by_account<F>(db: &Connection, account: Id, mut f: F) -> Result<()>
    where
        F: FnMut(Self),
    {
        match db
            .prepare("SELECT * FROM records WHERE account = ?")?
            .query_and_then([account], |row| Self::try_from(row))
        {
            Ok(iter) => {
                for entity in iter {
                    f(entity?);
                }
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }

    pub(crate) fn delete_by_account(
        db: &Connection,
        account: Id,
    ) -> Result<()> {
        db.execute(
            "DELETE FROM records
            WHERE account = :account",
            rusqlite::named_params! {":account": account},
        )?;
        Ok(())
    }
}

impl TryFrom<&rusqlite::Row<'_>> for Record {
    type Error = rusqlite::Error;

    fn try_from(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Record {
            id: row.get("id")?,
            account: row.get("account")?,
            amount: row.get::<&str, Money>("amount")?.into(),
            operation_date: row.get("operation_date")?,
            value_date: row.get("value_date")?,
            transaction_type: row.get("transaction_type")?,
            transaction_details: row.get("transaction_details")?,
            category: row.get("category")?,
            merchant: row.get("merchant")?,
        })
    }
}

impl Entity for Record {
    fn id(&self) -> Option<Id> {
        self.id
    }

    fn find(db: &Connection, id: Id) -> Result<Self> {
        let query = "SELECT * FROM records WHERE id = ? LIMIT 1;";
        let mut statement = db.prepare(query)?;
        match statement.query_row([id], |row| row.try_into()) {
            Ok(record) => Ok(record),
            Err(rusqlite::Error::QueryReturnedNoRows) => Err(Error::NotFound),
            Err(e) => Err(e.into()),
        }
    }

    fn save(&mut self, db: &Connection) -> Result<()> {
        use rusqlite::named_params;

        if let Some(id) = self.id() {
            let query = "
                UPDATE records
                SET
                    value_date = :value_date,
                    category = :category,
                    merchant = :merchant
                WHERE
                    id = :id";
            let mut statement = db.prepare(query)?;
            let params = named_params! {
                ":id": id,
                ":value_date": self.value_date,
                ":category": self.category,
                ":merchant": self.merchant
            };
            match statement.execute(params) {
                Ok(_) => Ok(()),
                Err(e) => Err(e.into()),
            }
        } else {
            let query = "
                INSERT INTO records (
                    account, amount,
                    operation_date, value_date,
                    transaction_type, transaction_details,
                    category,
                    merchant
                ) VALUES (
                    :account, :amount,
                    :operation_date, :value_date,
                    :transaction_type, :transaction_details,
                    :category,
                    :merchant
                )
                RETURNING id;";
            let mut statement = db.prepare(query)?;
            let params = named_params! {
                ":account": self.account,
                ":amount": Money::from(self.amount),
                ":operation_date": self.operation_date,
                ":value_date": self.value_date,
                ":transaction_type": self.transaction_type,
                ":transaction_details": self.transaction_details,
                ":category": self.category,
                ":merchant": self.merchant,
            };

            Ok(statement.query_row(params, |row| {
                self.id = row.get(0)?;
                Ok(())
            })?)
        }
    }
}

impl Upgrade for Record {
    fn upgrade_from(db: &Database, _version: &semver::Version) -> Result<()> {
        match db.execute(
            "CREATE TABLE IF NOT EXISTS records (
                    id INTEGER NOT NULL PRIMARY KEY,
                    account INTEGER NOT NULL,
                    amount BLOB NOT NULL,
                    operation_date TEXT NOT NULL,
                    value_date TEXT NOT NULL,
                    transaction_type TEXT,
                    transaction_details TEXT NOT NULL DEFAULT '',
                    category INTEGER,
                    merchant INTEGER
                );",
            (),
        ) {
            Ok(_) => Ok(()),
            Err(e) => Err(e.into()),
        }
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
