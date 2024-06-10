use crate::category::Category;
use crate::database::{
    Connection, Database, Entity, Error, Id, Result, Upgrade,
};

#[derive(Debug, Default)]
pub struct Merchant {
    id: Option<Id>,
    name: String,
    default_category: Option<Id>,
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
        self.default_category
    }

    /// Change the default category
    ///
    /// Passing a non-persisted category (i.e. without id) will instead reset
    /// the default_category to `None`
    pub fn set_default_category(&mut self, category: Option<&Category>) {
        self.default_category = category.and_then(|c| c.id());
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

    pub fn find_or_create_by_name<T: Into<String>>(
        db: &Connection,
        name: T,
    ) -> Result<Self> {
        let name_string: String = name.into();

        match Self::find_by_name(db, name_string.as_str()) {
            Err(Error::NotFound) => {
                let mut record = Self::new(name_string);
                record.save(db)?;
                Ok(record)
            }
            value => value,
        }
    }
}

impl TryFrom<&rusqlite::Row<'_>> for Merchant {
    type Error = rusqlite::Error;

    fn try_from(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Merchant {
            id: row.get("id")?,
            name: row.get("name")?,
            default_category: row.get("default_category")?,
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
                    default_category = :default_category
                WHERE
                    id = :id";
            let params = named_params! {
                ":id": id,
                ":name": self.name,
                ":default_category": self.default_category,
            };
            match db.prepare(query)?.execute(params) {
                Ok(_) => Ok(()),
                Err(e) => Err(e.into()),
            }
        } else {
            let query = "
                INSERT INTO merchants (
                    name,
                    default_category
                )
                VALUES (
                    :name,
                    :default_category
                )
                RETURNING id;";

            let params = named_params! {
                ":name": self.name,
                ":default_category": self.default_category,
            };

            Ok(db.prepare(query)?.query_row(params, |row| {
                self.id = row.get(0)?;
                Ok(())
            })?)
        }
    }
}

impl Upgrade for Merchant {
    fn upgrade_from(db: &Database, _version: &semver::Version) -> Result<()> {
        match db.execute(
            "CREATE TABLE IF NOT EXISTS merchants (
                id INTEGER NOT NULL PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                default_category INTEGER
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
        Merchant::setup(&db)?;

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
    fn find_or_create_by_name() -> Result<()> {
        let db = Database::memory()?;
        Merchant::setup(&db)?;

        assert!(matches!(
            Merchant::find_by_name(&db, "Chariot"),
            Err(Error::NotFound)
        ));

        let mut merchant = Merchant::new("Chariot");
        merchant.save(&db)?;

        assert_eq!(merchant.id(), Merchant::find_by_name(&db, "Chariot")?.id());

        merchant = Merchant::find_or_create_by_name(&db, "Uraidla Pub")?;
        assert_eq!(Some(Id::from(2)), merchant.id());

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

        // Check that default_category is correctly persisted
        merchant.save(&db)?;
        assert_eq!(
            category.id(),
            Merchant::find(&db, merchant.id().unwrap())?.default_category_id()
        );

        Ok(())
    }
}
