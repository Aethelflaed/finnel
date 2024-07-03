use chrono::{offset::Utc, DateTime};

use oxydized_money::{Amount, Currency, Decimal};

use crate::Database;
use db::{
    self as database, Connection, Entity, Error, Id, Result, Row, Upgrade,
};

use crate::category::Category;
use crate::merchant::Merchant;
use crate::transaction::{Direction, Mode};

mod new;
mod query;

pub use new::NewRecord;
pub use query::{FullRecord, QueryRecord};

use derive::{Entity, EntityDescriptor};

#[derive(Debug, Entity, EntityDescriptor)]
#[entity(table = "records")]
pub struct Record {
    id: Option<Id>,
    #[field(update = false)]
    account_id: Id,
    #[field(db_type = database::Decimal, update = false)]
    amount: Decimal,
    #[field(db_type = database::Currency, update = false)]
    currency: Currency,
    #[field(update = false)]
    operation_date: DateTime<Utc>,
    pub value_date: DateTime<Utc>,
    #[field(update = false)]
    direction: Direction,
    #[field(update = false)]
    mode: Mode,
    details: String,
    category_id: Option<Id>,
    merchant_id: Option<Id>,
}

impl Record {
    pub fn amount(&self) -> Amount {
        Amount(self.amount, self.currency)
    }

    pub fn operation_date(&self) -> DateTime<Utc> {
        self.operation_date
    }

    pub fn direction(&self) -> Direction {
        self.direction
    }

    pub fn mode(&self) -> Mode {
        self.mode.clone()
    }

    pub fn details(&self) -> &str {
        self.details.as_str()
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
            .query_and_then([account_id], |row| Self::try_from(&Row::from(row)))
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
            amount: Decimal::new(314, 2),
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
        assert_eq!(Amount(Decimal::new(314, 2), Currency::USD), record.amount());

        let mut category = Category::new("category");
        category.save(&db)?;

        record.set_category(Some(&category));
        record.save(&db)?;

        let record = Record::find(&db, record.id().unwrap())?;
        assert_eq!(category.id(), record.category_id());

        Ok(())
    }
}
