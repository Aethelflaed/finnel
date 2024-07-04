use crate::Database;
use db::{Connection, Entity, Error, Id, Result, Row, Upgrade};

use derive::{Entity, EntityDescriptor};

mod query;
pub use query::QueryCategory;

#[derive(Debug, Default, Entity, EntityDescriptor)]
#[entity(table = "categories")]
pub struct Category {
    id: Option<Id>,
    pub name: String,
}

impl Category {
    pub fn new<T: Into<String>>(name: T) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    pub fn find_by_name(db: &Connection, name: &str) -> Result<Self> {
        let query = "SELECT * FROM categories WHERE name = ? LIMIT 1;";
        let mut statement = db.prepare(query)?;

        match statement.query_row([name], |row| Self::try_from(&Row::from(row)))
        {
            Ok(record) => Ok(record),
            Err(rusqlite::Error::QueryReturnedNoRows) => Err(Error::NotFound),
            Err(e) => Err(e.into()),
        }
    }
}

impl Upgrade<Category> for Database {
    fn upgrade_from(&self, _version: &semver::Version) -> Result<()> {
        match self.execute(
            "
                CREATE TABLE IF NOT EXISTS categories (
                    id INTEGER NOT NULL PRIMARY KEY,
                    name TEXT NOT NULL UNIQUE
                );
            ",
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

    #[test]
    fn crud() -> Result<()> {
        let db = Database::memory()?;
        db.setup()?;

        let mut category = Category::new("Uraidla Pub");
        assert_eq!(None, category.id());
        category.save(&db)?;
        assert_eq!(Some(Id::from(1)), category.id());

        assert_eq!("Uraidla Pub", category.name);
        category.name = "Chariot".to_string();
        category.save(&db)?;
        assert_eq!("Chariot", Category::find(&db, Id::from(1))?.name);

        Ok(())
    }
}
