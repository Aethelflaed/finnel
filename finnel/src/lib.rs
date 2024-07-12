#[cfg(test)]
pub mod test;

pub mod db;

pub mod result;

pub mod account;
pub mod category;
pub mod merchant;
pub mod record;

pub mod schema;
use diesel::prelude::*;

pub mod essentials {
    pub use crate::result::{Error, OptionalExtension, Result};
    pub use oxydized_money::{Amount, Currency, Decimal};
    pub type Conn = diesel::sqlite::SqliteConnection;
}
pub use essentials::*;

pub mod prelude {
    pub use crate::essentials::*;

    pub use crate::{
        account::Account, category::Category, merchant::Merchant,
        record::Record,
    };
}

#[derive(
    derive_more::From,
    derive_more::Into,
    derive_more::Deref,
    derive_more::DerefMut,
)]
pub struct Database(SqliteConnection);

use diesel_migrations::{
    embed_migrations, EmbeddedMigrations, MigrationHarness,
};
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

impl Database {
    pub fn open<T: AsRef<std::path::Path>>(path: T) -> Result<Self> {
        Ok(Database(SqliteConnection::establish(
            &path.as_ref().to_string_lossy(),
        )?))
    }

    pub fn memory() -> Result<Self> {
        Self::open(":memory:")
    }

    pub fn setup(&mut self) -> Result<()> {
        self.run_pending_migrations(MIGRATIONS)?;

        Ok(())
    }
}
