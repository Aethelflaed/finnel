#![cfg(test)]

use crate::prelude::*;

use anyhow::Result;

pub mod prelude {
    pub use crate::test::{self, Reloadable};
    pub use anyhow::Result;
    pub use pretty_assertions::{assert_eq, assert_ne};
}

pub trait Reloadable {
    fn reload(&mut self, conn: &mut Conn) -> Result<&mut Self>;
}

macro_rules! reloadable {
    ($($struct:ident),*) => {
        $(impl Reloadable for $struct {
            fn reload(&mut self, conn: &mut Conn) -> Result<&mut Self> {
                *self = Self::find(conn, self.id)?;
                Ok(self)
            }
        })*
    };
}

reloadable!(Account, Category, Merchant, Record);

pub fn db() -> Result<Conn> {
    let mut db = crate::Database::memory()?;
    db.setup()?;
    Ok(db.into())
}

pub fn account(conn: &mut Conn, name: &str) -> Result<Account> {
    Ok(crate::account::NewAccount::new(name).save(conn)?)
}

pub fn category(conn: &mut Conn, name: &str) -> Result<Category> {
    Ok(crate::category::NewCategory::new(name).save(conn)?)
}

pub fn merchant(conn: &mut Conn, name: &str) -> Result<Merchant> {
    Ok(crate::merchant::NewMerchant::new(name).save(conn)?)
}

pub fn record(conn: &mut Conn, account: &Account) -> Result<Record> {
    Ok(crate::record::NewRecord::new(account).save(conn)?)
}
