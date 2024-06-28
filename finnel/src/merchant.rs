use crate::category::Category;
use crate::Database;
use db::{Connection, Entity, Error, Id, Result, Upgrade};

#[derive(Debug, Default)]
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

        match statement.query_row([name], |row| row.try_into()) {
            Ok(record) => Ok(record),
            Err(rusqlite::Error::QueryReturnedNoRows) => Err(Error::NotFound),
            Err(e) => Err(e.into()),
        }
    }
}

impl TryFrom<&rusqlite::Row<'_>> for Merchant {
    type Error = rusqlite::Error;

    fn try_from(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Merchant {
            id: row.get("id")?,
            name: row.get("name")?,
            default_category_id: row.get("default_category_id")?,
        })
    }
}

impl Entity for Merchant {
    fn id(&self) -> Option<Id> {
        self.id
    }

    fn find(db: &Connection, id: Id) -> Result<Self> {
        let query = "SELECT * FROM merchants WHERE id = ? LIMIT 1;";
        match db.prepare(query)?.query_row([id], |row| row.try_into()) {
            Ok(record) => Ok(record),
            Err(rusqlite::Error::QueryReturnedNoRows) => Err(Error::NotFound),
            Err(e) => Err(e.into()),
        }
    }

    fn save(&mut self, db: &Connection) -> Result<()> {
        use rusqlite::named_params;

        if let Some(id) = self.id() {
            let query = "
                UPDATE merchants
                SET
                    name = :name,
                    default_category_id = :default_category_id
                WHERE
                    id = :id";
            let params = named_params! {
                ":id": id,
                ":name": self.name,
                ":default_category_id": self.default_category_id,
            };
            match db.prepare(query)?.execute(params) {
                Ok(_) => Ok(()),
                Err(e) => Err(e.into()),
            }
        } else {
            let query = "
                INSERT INTO merchants (
                    name,
                    default_category_id
                )
                VALUES (
                    :name,
                    :default_category_id
                )
                RETURNING id;";

            let params = named_params! {
                ":name": self.name,
                ":default_category_id": self.default_category_id,
            };

            Ok(db.prepare(query)?.query_row(params, |row| {
                self.id = row.get(0)?;
                Ok(())
            })?)
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
