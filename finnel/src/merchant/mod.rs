use crate::database::{Database, Connection, Entity, Error, Result, Upgrade};

pub use crate::database::Id;

#[derive(Debug, Default)]
pub struct Merchant {
    id: Option<Id>,
    name: String,
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
        })
    }
}

impl Entity for Merchant {
    fn id(&self) -> Option<Id> {
        self.id
    }

    fn find(db: &Connection, id: Id) -> Result<Self> {
        let query = "SELECT * FROM merchants WHERE id = ? LIMIT 1;";
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
                UPDATE merchants
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
                INSERT INTO merchants (
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

impl Upgrade for Merchant {
    fn upgrade_from(db: &Database, _version: &semver::Version) -> Result<()> {
        match db.execute(
            "CREATE TABLE IF NOT EXISTS merchants (
                id INTEGER NOT NULL PRIMARY KEY,
                name TEXT NOT NULL UNIQUE
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
    use pretty_assertions::assert_eq;

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
}
