use chrono::{offset::Utc, DateTime};
use oxydized_money::{Currency, Decimal};

use crate::account::Account;
use crate::record::Record;
use crate::transaction::{Direction, Mode};
use db::{Connection, Entity, Error, Id, Result};

#[derive(Clone, Debug)]
pub struct NewRecord {
    pub account_id: Option<Id>,
    pub amount: Decimal,
    pub currency: Currency,
    pub operation_date: DateTime<Utc>,
    pub value_date: DateTime<Utc>,
    pub direction: Direction,
    pub mode: Mode,
    pub details: String,
    pub category_id: Option<Id>,
    pub merchant_id: Option<Id>,
}

impl Default for NewRecord {
    fn default() -> Self {
        let date = Utc::now();

        Self {
            account_id: None,
            amount: Decimal::ZERO,
            currency: Currency::EUR,
            operation_date: date,
            value_date: date,
            direction: Direction::Debit,
            mode: Mode::Direct,
            details: String::new(),
            category_id: None,
            merchant_id: None,
        }
    }
}

fn invalid(msg: &str) -> Error {
    Error::Invalid(msg.to_string())
}

impl NewRecord {
    pub fn new(account: &Account) -> Self {
        Self {
            account_id: account.id(),
            currency: account.currency,
            ..Default::default()
        }
    }

    pub fn save(&mut self, db: &Connection) -> Result<Record> {
        let Some(account_id) = self.account_id else {
            return Err(invalid("Account not provided"));
        };
        let account = Account::find(db, account_id)?;
        if self.currency != account.currency() {
            return Err(invalid("Currency mismatch"));
        }

        let mut record = Record {
            id: None,
            account_id,
            amount: self.amount,
            currency: self.currency,
            operation_date: self.operation_date,
            value_date: self.value_date,
            direction: self.direction,
            mode: self.mode.clone(),
            details: self.details.clone(),
            category_id: self.category_id,
            merchant_id: self.merchant_id,
        };

        record.save(db)?;

        Ok(record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::prelude::{assert_eq, Result, *};

    use crate::{Account, Amount};

    fn error_contains_msg<E, S>(error: E, message: S) -> bool
    where
        E: std::error::Error,
        S: AsRef<str>,
    {
        use predicates::{str::contains, Predicate};

        contains(message.as_ref()).eval(format!("{:?}", error).as_str())
    }

    #[test]
    fn validation_and_creation() -> Result<()> {
        let db = test::db()?;

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
        let record = new_record.save(&db)?;
        assert_eq!(record.account_id, account.id().unwrap());
        assert_eq!(
            Amount(Decimal::new(314, 2), Currency::USD),
            record.amount()
        );

        Ok(())
    }
}
