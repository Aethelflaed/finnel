#![cfg(test)]

use crate::{
    Account, Category, Connection, Database, Entity, Merchant, Record,
};
use anyhow::Result;

pub mod prelude {
    pub use crate::test::{self, Reload};
    pub use anyhow::Result;
    pub use pretty_assertions::{assert_eq, assert_ne};
}

pub trait Reload: Entity {
    fn reload(&mut self, db: &Connection) -> Result<&mut Self> {
        *self = Self::find(
            db,
            self.id()
                .ok_or(anyhow::anyhow!("Can't reload entity without id"))?,
        )?;
        Ok(self)
    }
}

impl Reload for Account {}
impl Reload for Record {}
impl Reload for Merchant {}
impl Reload for Category {}

pub fn db() -> Result<Database> {
    let db = Database::memory()?;
    db.setup()?;
    Ok(db)
}

pub fn account(db: &Connection, name: &str) -> Result<Account> {
    let mut account = Account::new(name);
    account.save(db)?;
    Ok(account)
}

pub fn category(db: &Connection, name: &str) -> Result<Category> {
    let mut category = Category::new(name);
    category.save(db)?;
    Ok(category)
}

pub fn merchant(db: &Connection, name: &str) -> Result<Merchant> {
    let mut merchant = Merchant::new(name);
    merchant.save(db)?;
    Ok(merchant)
}

pub fn record(db: &Connection, account: &Account) -> Result<Record> {
    let mut record = crate::record::NewRecord::new(account);
    Ok(record.save(db)?)
}
