use crate::Database;
use db::{Connection, Entity, Error, Id, Result, Upgrade};

#[derive(Debug, Default)]
pub struct Category {
    id: Option<Id>,
    name: String,
}

impl Category {
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

    pub fn find_by_name(db: &Connection, name: &str) -> Result<Self> {
        let query = "SELECT * FROM categories WHERE name = ? LIMIT 1;";
        let mut statement = db.prepare(query)?;

        match statement.query_row([name], |row| row.try_into()) {
            Ok(record) => Ok(record),
            Err(rusqlite::Error::QueryReturnedNoRows) => Err(Error::NotFound),
            Err(e) => Err(e.into()),
        }
    }
}

impl TryFrom<&rusqlite::Row<'_>> for Category {
    type Error = rusqlite::Error;

    fn try_from(row: &rusqlite::Row) -> rusqlite::Result<Self> {
        Ok(Category {
            id: row.get("id")?,
            name: row.get("name")?,
        })
    }
}

impl Entity for Category {
    fn id(&self) -> Option<Id> {
        self.id
    }

    fn find(db: &Connection, id: Id) -> Result<Self> {
        let query = "SELECT * FROM categories WHERE id = ? LIMIT 1;";
        let mut statement = db.prepare(query)?;
        match statement.query_row([id], |row| row.try_into()) {
            Ok(record) => Ok(record),
            Err(rusqlite::Error::QueryReturnedNoRows) => Err(Error::NotFound),
            Err(e) => Err(e.into()),
        }
    }

    fn save(&mut self, db: &Connection) -> Result<()> {
        use rusqlite::named_params;

        if let Some(id) = self.id() {
            let query = "
                UPDATE categories
                SET
                    name = :name
                WHERE
                    id = :id";
            let mut statement = db.prepare(query)?;
            match statement
                .execute(named_params! {":id": id, ":name": self.name})
            {
                Ok(_) => Ok(()),
                Err(e) => Err(e.into()),
            }
        } else {
            let query = "
                INSERT INTO categories (
                    name
                )
                VALUES (
                    :name
                )
                RETURNING id;";
            let mut statement = db.prepare(query)?;

            Ok(statement.query_row(
                &[(":name", self.name.as_str())],
                |row| {
                    self.id = row.get(0)?;
                    Ok(())
                },
            )?)
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

        assert_eq!("Uraidla Pub", category.name());
        category.set_name("Chariot");
        category.save(&db)?;
        assert_eq!("Chariot", Category::find(&db, Id::from(1))?.name());

        Ok(())
    }
}
