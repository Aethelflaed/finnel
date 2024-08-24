#![cfg(test)]

use crate::prelude::*;

use anyhow::Result;

pub mod prelude {
    pub use crate::essentials::{OptionalExtension, *};
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

reloadable!(Account, Category, Merchant, Record, Report);

pub fn db() -> Result<Conn> {
    let mut db = crate::Database::memory()?;
    db.setup()?;
    Ok(db.into())
}

macro_rules! setter {
    ($object:ident) => {};
    ($object:ident, $field:ident: $value:expr) => {
        $object.$field = $value;
    };
    ($object:ident, $field:ident: $value:expr, $($tail:tt)*) => {
        $object.$field = $value;
        setter!($object, $($tail)*);
    };
}
pub(crate) use setter;

macro_rules! account {
    ($conn:ident, $name:expr) => {
        crate::account::NewAccount::new($name).save($conn)?
    };
    ($conn:ident, $name:expr, $($tail:tt)*) => {
        {
            let mut object = crate::account::NewAccount::new($name);
            test::setter!(object, $($tail)*);
            object.save($conn)?
        }
    };
}

macro_rules! category {
    ($conn:ident, $name:expr) => {
        crate::category::NewCategory::new($name).save($conn)?
    };
    ($conn:ident, $name:expr, $($tail:tt)*) => {
        {
            let mut object = crate::category::NewCategory::new($name);
            test::setter!(object, $($tail)*);
            object.save($conn)?
        }
    };
}

macro_rules! merchant {
    ($conn:ident, $name:expr) => {
        crate::merchant::NewMerchant::new($name).save($conn)?
    };
    ($conn:ident, $name:expr, $($tail:tt)*) => {
        {
            let mut object = crate::merchant::NewMerchant::new($name);
            test::setter!(object, $($tail)*);
            object.save($conn)?
        }
    };
}

macro_rules! record {
    ($conn:ident, $account:expr) => {
        crate::record::NewRecord::new($account).save($conn)?
    };
    ($conn:ident, $account:expr, $($tail:tt)*) => {
        {
            let mut object = crate::record::NewRecord::new($account);
            test::setter!(object, $($tail)*);
            object.save($conn)?
        }
    };
}

pub(crate) use account;
pub(crate) use category;
pub(crate) use merchant;
pub(crate) use record;
