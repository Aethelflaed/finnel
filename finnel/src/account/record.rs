use chrono::{offset::Utc, DateTime};

use oxydized_money::{Amount, Currency, Decimal};

use crate::database::{
    self, Connection, Database, Entity, Error, Id, Result, Upgrade,
};

use crate::account::Account;
use crate::category::Category;
use crate::merchant::Merchant;
use crate::transaction::{Direction, Mode};

#[derive(Debug)]
pub struct Record {
    id: Option<Id>,
    account: Id,
    amount: Decimal,
    currency: Currency,
    operation_date: DateTime<Utc>,
    value_date: DateTime<Utc>,
    direction: Direction,
    mode: Mode,
    details: String,
    category: Option<Id>,
    merchant: Option<Id>,
}

#[derive(Debug)]
pub struct NewRecord {
    pub account: Option<Id>,
    pub amount: Decimal,
    pub currency: Currency,
    pub operation_date: DateTime<Utc>,
    pub value_date: DateTime<Utc>,
    pub direction: Direction,
    pub mode: Mode,
    pub details: String,
    pub category: Option<Id>,
    pub merchant: Option<Id>,
}

impl Default for NewRecord {
    fn default() -> Self {
        let date = Utc::now();

        Self {
            account: None,
            amount: Decimal::ZERO,
            currency: Currency::EUR,
            operation_date: date,
            value_date: date,
            direction: Direction::Debit,
            mode: Mode::Direct,
            details: String::new(),
            category: None,
            merchant: None,
        }
    }
}

fn invalid(msg: &str) -> Error {
    Error::Invalid(msg.to_string())
}

impl NewRecord {
    pub fn save(&mut self, db: &Connection) -> Result<Record> {
        let Some(account_id) = self.account else {
            return Err(invalid("Account not provided"));
        };
        let account = Account::find(db, account_id)?;
        if self.currency != account.currency() {
            return Err(invalid("Currency mismatch"));
        }

        let mut record = Record {
            id: None,
            account: account_id,
            amount: self.amount,
            currency: self.currency,
            operation_date: self.operation_date,
            value_date: self.value_date,
            direction: self.direction,
            mode: self.mode.clone(),
            details: self.details.clone(),
            category: self.category,
            merchant: self.merchant,
        };

        record.save(db)?;

        Ok(record)
    }
}

impl Record {
    pub fn amount(&self) -> Amount {
        Amount(self.amount, self.currency)
    }

    pub fn set_value_date(&mut self, value: DateTime<Utc>) {
        self.value_date = value;
    }

    pub fn category_id(&self) -> Option<Id> {
        self.category
    }

    pub fn set_category(&mut self, category: Option<&Category>) {
        self.category = category.and_then(|c| c.id());
    }

    pub fn merchant_id(&self) -> Option<Id> {
        self.merchant
    }
    pub fn set_merchant(&mut self, merchant: Option<&Merchant>) {
        self.merchant = merchant.and_then(|m| m.id());
    }

    pub fn by_account_id<F>(
        db: &Connection,
        account: Id,
        mut f: F,
    ) -> Result<()>
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

    pub(crate) fn delete_by_account_id(
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
            amount: row.get::<&str, database::Decimal>("amount")?.into(),
            currency: row.get::<&str, database::Currency>("currency")?.into(),
            operation_date: row.get("operation_date")?,
            value_date: row.get("value_date")?,
            direction: row.get("direction")?,
            mode: row.get("mode")?,
            details: row.get("details")?,
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
                    account, amount, currency,
                    operation_date, value_date,
                    direction, mode, details,
                    category,
                    merchant
                ) VALUES (
                    :account, :amount, :currency,
                    :operation_date, :value_date,
                    :direction, :mode, :details,
                    :category,
                    :merchant
                )
                RETURNING id;";
            let mut statement = db.prepare(query)?;
            let params = named_params! {
                ":account": self.account,
                ":amount": database::Decimal::from(self.amount),
                ":currency": database::Currency::from(self.currency),
                ":operation_date": self.operation_date,
                ":value_date": self.value_date,
                ":direction": self.direction,
                ":mode": self.mode,
                ":details": self.details,
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
                    amount TEXT NOT NULL,
                    currency TEXT NOT NULL,
                    operation_date TEXT NOT NULL,
                    value_date TEXT NOT NULL,
                    direction TEXT NOT NULL DEFAULT 'Debit',
                    mode TEXT NOT NULL DEFAULT 'Direct',
                    details TEXT NOT NULL DEFAULT '',
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

    fn error_contains_msg<E, S>(error: E, message: S) -> bool
    where
        E: std::error::Error,
        S: AsRef<str>
    {
        use predicates::{Predicate, str::contains};

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

        new_record.account = account.id();

        let error = new_record.save(&db).unwrap_err();
        assert!(error_contains_msg(error, "Currency mismatch"));

        new_record.currency = account.currency;
        let mut record = new_record.save(&db)?;
        assert_eq!(record.account, account.id().unwrap());
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
