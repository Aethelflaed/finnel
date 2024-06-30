use chrono::{offset::Utc, DateTime};

use oxydized_money::{Amount, Currency, Decimal};

use crate::Database;
use db::{self as database, Connection, Entity, Error, Id, Result, Upgrade};

use crate::category::Category;
use crate::merchant::Merchant;
use crate::transaction::{Direction, Mode};

mod new;
mod query;

pub use new::NewRecord;
pub use query::QueryRecord;

#[derive(Debug)]
pub struct Record {
    id: Option<Id>,
    account_id: Id,
    amount: Decimal,
    currency: Currency,
    operation_date: DateTime<Utc>,
    value_date: DateTime<Utc>,
    direction: Direction,
    mode: Mode,
    details: String,
    category_id: Option<Id>,
    merchant_id: Option<Id>,
}

impl Record {
    pub fn amount(&self) -> Amount {
        Amount(self.amount, self.currency)
    }

    pub fn set_value_date(&mut self, value: DateTime<Utc>) {
        self.value_date = value;
    }

    pub fn category_id(&self) -> Option<Id> {
        self.category_id
    }

    pub fn set_category(&mut self, category: Option<&Category>) {
        self.category_id = category.and_then(Entity::id);
    }

    pub fn merchant_id(&self) -> Option<Id> {
        self.merchant_id
    }
    pub fn set_merchant(&mut self, merchant: Option<&Merchant>) {
        self.merchant_id = merchant.and_then(Entity::id);
    }
}

impl Record {
    pub fn by_account_id<F>(
        db: &Connection,
        account_id: Id,
        mut f: F,
    ) -> Result<()>
    where
        F: FnMut(Self),
    {
        match db
            .prepare("SELECT * FROM records WHERE account_id = ?")?
            .query_and_then([account_id], |row| Self::try_from(row))
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

    pub(crate) fn delete_by_account_id(
        db: &Connection,
        account_id: Id,
    ) -> Result<()> {
        db.execute(
            "DELETE FROM records
            WHERE account_id = :account_id",
            rusqlite::named_params! {":account_id": account_id},
        )?;
        Ok(())
    }
}

impl TryFrom<&rusqlite::Row<'_>> for Record {
    type Error = rusqlite::Error;

    fn try_from(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Record {
            id: row.get("id")?,
            account_id: row.get("account_id")?,
            amount: row.get::<&str, database::Decimal>("amount")?.into(),
            currency: row.get::<&str, database::Currency>("currency")?.into(),
            operation_date: row.get("operation_date")?,
            value_date: row.get("value_date")?,
            direction: row.get("direction")?,
            mode: row.get("mode")?,
            details: row.get("details")?,
            category_id: row.get("category_id")?,
            merchant_id: row.get("merchant_id")?,
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
                    category_id = :category_id,
                    merchant_id = :merchant_id
                WHERE
                    id = :id";
            let mut statement = db.prepare(query)?;
            let params = named_params! {
                ":id": id,
                ":value_date": self.value_date,
                ":category_id": self.category_id,
                ":merchant_id": self.merchant_id
            };
            match statement.execute(params) {
                Ok(_) => Ok(()),
                Err(e) => Err(e.into()),
            }
        } else {
            let query = "
                INSERT INTO records (
                    account_id, amount, currency,
                    operation_date, value_date,
                    direction, mode, details,
                    category_id,
                    merchant_id
                ) VALUES (
                    :account_id, :amount, :currency,
                    :operation_date, :value_date,
                    :direction, :mode, :details,
                    :category_id,
                    :merchant_id
                )
                RETURNING id;";
            let mut statement = db.prepare(query)?;
            let params = named_params! {
                ":account_id": self.account_id,
                ":amount": database::Decimal::from(self.amount),
                ":currency": database::Currency::from(self.currency),
                ":operation_date": self.operation_date,
                ":value_date": self.value_date,
                ":direction": self.direction,
                ":mode": self.mode,
                ":details": self.details,
                ":category_id": self.category_id,
                ":merchant_id": self.merchant_id,
            };

            Ok(statement.query_row(params, |row| {
                self.id = row.get(0)?;
                Ok(())
            })?)
        }
    }
}

impl Upgrade<Record> for Database {
    fn upgrade_from(&self, _version: &semver::Version) -> Result<()> {
        match self.execute(
            "CREATE TABLE IF NOT EXISTS records (
                    id INTEGER NOT NULL PRIMARY KEY,
                    account_id INTEGER NOT NULL,
                    amount INTEGER NOT NULL,
                    currency TEXT NOT NULL,
                    operation_date TEXT NOT NULL,
                    value_date TEXT NOT NULL,
                    direction TEXT NOT NULL DEFAULT 'Debit',
                    mode TEXT NOT NULL DEFAULT 'Direct',
                    details TEXT NOT NULL DEFAULT '',
                    category_id INTEGER,
                    merchant_id INTEGER
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

    use crate::Account;

    fn error_contains_msg<E, S>(error: E, message: S) -> bool
    where
        E: std::error::Error,
        S: AsRef<str>,
    {
        use predicates::{str::contains, Predicate};

        contains(message.as_ref()).eval(format!("{:?}", error).as_str())
    }

    #[test]
    fn crud() -> anyhow::Result<()> {
        let db = Database::memory()?;
        db.setup()?;

        let mut account = Account::new("Cash");
        account.currency = Currency::USD;

        let mut new_record = NewRecord {
            amount: Decimal::from_str_exact("3.14")?,
            ..Default::default()
        };
        let error = new_record.save(&db).unwrap_err();
        assert!(error_contains_msg(error, "Account not provided"));

        account.save(&db)?;

        new_record.account_id = account.id();

        let error = new_record.save(&db).unwrap_err();
        assert!(error_contains_msg(error, "Currency mismatch"));

        new_record.currency = account.currency();
        let mut record = new_record.save(&db)?;
        assert_eq!(record.account_id, account.id().unwrap());
        assert_eq!(Currency::USD, record.currency);

        let mut category = Category::new("category");
        category.save(&db)?;

        record.set_category(Some(&category));
        record.save(&db)?;

        let record = Record::find(&db, record.id().unwrap())?;
        assert_eq!(category.id(), record.category_id());

        Ok(())
    }
}
