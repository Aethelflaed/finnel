use crate::category::Category;
use crate::Database;
use db::{Connection, Entity, Error, Id, Result, Row, Upgrade};

use derive::{Entity, EntityDescriptor};

#[derive(Debug, Default, Entity, EntityDescriptor)]
#[entity(table = "merchants")]
pub struct Merchant {
    id: Option<Id>,
    name: String,
    default_category_id: Option<Id>,
}

impl Merchant {
    pub fn new<T: Into<String>>(name: T) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn set_name<T: Into<String>>(&mut self, name: T) {
        self.name = name.into();
    }

    pub fn default_category_id(&self) -> Option<Id> {
        self.default_category_id
    }

    /// Change the default category
    ///
    /// Passing a non-persisted category (i.e. without id) will instead reset
    /// the default_category_id to `None`
    pub fn set_default_category(&mut self, category: Option<&Category>) {
        self.default_category_id = category.and_then(Entity::id);
    }

    pub fn find_by_name(db: &Connection, name: &str) -> Result<Self> {
        let query = "SELECT * FROM merchants WHERE name = ? LIMIT 1;";
        let mut statement = db.prepare(query)?;

        match statement.query_row([name], |row| Self::try_from(&Row::from(row)))
        {
            Ok(record) => Ok(record),
            Err(rusqlite::Error::QueryReturnedNoRows) => Err(Error::NotFound),
            Err(e) => Err(e.into()),
        }
    }
}

impl Upgrade<Merchant> for Database {
    fn upgrade_from(&self, _version: &semver::Version) -> Result<()> {
        match self.execute(
            "CREATE TABLE IF NOT EXISTS merchants (
                id INTEGER NOT NULL PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                default_category_id INTEGER
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
    use pretty_assertions::{assert_eq, assert_ne};

    #[test]
    fn crud() -> Result<()> {
        let db = Database::memory()?;
        db.setup()?;

        let mut merchant = Merchant::new("Uraidla Pub");
        assert_eq!(None, merchant.id());
        merchant.save(&db)?;
        assert_eq!(Some(Id::from(1)), merchant.id());

        assert_eq!("Uraidla Pub", merchant.name());
        merchant.set_name("Chariot");
        merchant.save(&db)?;
        assert_eq!("Chariot", Merchant::find(&db, Id::from(1))?.name());

        Ok(())
    }

    #[test]
    fn default_category() -> Result<()> {
        let db = Database::memory()?;
        db.setup()?;

        let mut category = Category::new("foo");
        let mut merchant = Merchant::new("bar");

        merchant.set_default_category(Some(&category));
        assert_eq!(None, merchant.default_category_id());

        category.save(&db)?;
        merchant.set_default_category(Some(&category));
        assert_ne!(None, category.id());
        assert_eq!(category.id(), merchant.default_category_id());

        // Check that default_category_id is correctly persisted
        merchant.save(&db)?;
        assert_eq!(
            category.id(),
            Merchant::find(&db, merchant.id().unwrap())?.default_category_id()
        );

        Ok(())
    }
}
