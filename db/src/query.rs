use std::marker::PhantomData;

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

    // TODO: add count()

    fn statement<'a>(&'a self, db: &'a Connection) -> Result<Statement<'a, T>> {
        self.valid()?;

        Ok(Statement {
            stmt: db.prepare(self.query().as_str())?,
            params: self.params(),
            phantom: PhantomData::<T>,
        })
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

pub struct Statement<'a, T>
where
    T: TryFrom<&'a Row<'a>, Error = rusqlite::Error>,
{
    stmt: rusqlite::Statement<'a>,
    params: Vec<(&'a str, &'a dyn ToSql)>,
    phantom: PhantomData<T>,
}

impl<T> Statement<'_, T>
where
    T: for<'a> TryFrom<&'a Row<'a>, Error = rusqlite::Error>,
{
    pub fn iter(&mut self) -> Result<Iter<'_, T>> {
        Ok(Iter {
            rows: self.stmt.query(self.params.as_slice())?,
            phantom: Default::default(),
        })
    }
}

pub struct Iter<'a, T>
where
    T: TryFrom<&'a Row<'a>, Error = rusqlite::Error>,
{
    rows: rusqlite::Rows<'a>,
    phantom: PhantomData<T>,
}

impl<T> Iterator for Iter<'_, T>
where
    T: for<'a> TryFrom<&'a Row<'a>, Error = rusqlite::Error>,
{
    type Item = rusqlite::Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.rows.next().transpose().map(move |row_result| {
            row_result.and_then(|row| T::try_from(&Row::from(row)))
        })
    }
}
