use crate::{entity::EntityDescriptor, Connection, Result, Row};
use rusqlite::ToSql;

pub trait Query<T, U = T>
where
    T: for<'a> TryFrom<&'a Row<'a>, Error = rusqlite::Error>,
    U: EntityDescriptor,
{
    fn query(&self) -> String;
    fn params(&self) -> Vec<(&str, &dyn ToSql)>;

    fn valid(&self) -> Result<()> {
        Ok(())
    }

    fn for_each<F>(&self, db: &Connection, mut f: F) -> Result<()>
    where
        F: FnMut(T),
    {
        self.valid()?;

        match db
            .prepare(self.query().as_str())?
            .query_and_then(self.params().as_slice(), |row| {
                T::try_from(&Row::from(row))
            }) {
            Ok(iter) => {
                for entity in iter {
                    f(entity?);
                }
                Ok(())
            }
            Err(e) => Err(e.into()),
        }
    }
}
