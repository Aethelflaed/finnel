use crate::{Connection, Result};
use rusqlite::ToSql;

pub trait Query<T>
where
    T: for<'a> TryFrom<&'a rusqlite::Row<'a>, Error = rusqlite::Error>,
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
            .query_and_then(self.params().as_slice(), |row| T::try_from(row))
        {
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
