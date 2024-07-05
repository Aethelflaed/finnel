pub mod account;
pub mod category;
pub mod merchant;
pub mod record;
pub mod transaction;

#[cfg(test)]
pub mod test;

pub use account::Account;
pub use category::Category;
pub use merchant::Merchant;
pub use record::Record;

pub use db::{Connection, DatabaseTrait, Entity, Error, Id, Query};

pub use oxydized_money::{Amount, Currency, Decimal};

#[derive(
    derive_more::From,
    derive_more::Into,
    derive_more::Deref,
    derive_more::DerefMut,
)]
pub struct Database(Connection);

use db::Result;

impl Database {
    pub fn open<T: AsRef<std::path::Path>>(path: T) -> Result<Self> {
        <Self as DatabaseTrait>::open(path)
    }

    pub fn memory() -> Result<Self> {
        <Self as DatabaseTrait>::memory()
    }

    pub fn setup(&self) -> Result<()> {
        DatabaseTrait::setup(self)
    }
}

impl DatabaseTrait for Database {
    fn upgrade_from(&self, version: &semver::Version) -> Result<()> {
        use db::Upgrade;

        Upgrade::<Category>::upgrade_from(self, version)?;
        Upgrade::<Merchant>::upgrade_from(self, version)?;
        Upgrade::<Account>::upgrade_from(self, version)?;
        Upgrade::<Record>::upgrade_from(self, version)?;

        Ok(())
    }
}
