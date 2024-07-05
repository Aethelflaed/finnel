use rusqlite::{
    types::{FromSql, ValueRef},
    Result,
};

pub enum Row<'a> {
    Prefixed(&'a PrefixedRow<'a>),
    Standard(&'a rusqlite::Row<'a>),
}

impl<'a> From<&'a rusqlite::Row<'a>> for Row<'a> {
    fn from(row: &'a rusqlite::Row<'a>) -> Self {
        Row::<'a>::Standard(row)
    }
}

impl<'a> From<&'a PrefixedRow<'a>> for Row<'a> {
    fn from(row: &'a PrefixedRow<'a>) -> Self {
        Row::<'a>::Prefixed(row)
    }
}

pub struct PrefixedRow<'a> {
    row: &'a Row<'a>,
    prefix: &'a str,
}

impl PrefixedRow<'_> {
    fn get<T: FromSql>(&self, idx: &str) -> Result<T> {
        self.row.get(format!("{}{}", self.prefix, idx).as_str())
    }

    fn get_unwrap<T: FromSql>(&self, idx: &str) -> T {
        self.row
            .get_unwrap(format!("{}{}", self.prefix, idx).as_str())
    }

    fn get_ref(&self, idx: &str) -> Result<ValueRef<'_>> {
        self.row.get_ref(format!("{}{}", self.prefix, idx).as_str())
    }

    fn get_ref_unwrap(&self, idx: &str) -> ValueRef<'_> {
        self.row
            .get_ref_unwrap(format!("{}{}", self.prefix, idx).as_str())
    }
}

impl Row<'_> {
    pub fn get_maybe_prefixed<T: FromSql>(
        &self,
        idx: &str,
        prefix: &str,
    ) -> Result<T> {
        match self {
            Row::Prefixed(row) => row.get(idx),
            Row::Standard(row) => match row.get::<&str, T>(idx) {
                Ok(r) => Ok(r),
                Err(_) => self.with_prefix(prefix, |row| row.get(idx)),
            },
        }
    }

    pub fn get<T: FromSql>(&self, idx: &str) -> Result<T> {
        match self {
            Row::Prefixed(row) => row.get(idx),
            Row::Standard(row) => row.get(idx),
        }
    }

    pub fn get_unwrap<T: FromSql>(&self, idx: &str) -> T {
        match self {
            Row::Prefixed(row) => row.get_unwrap(idx),
            Row::Standard(row) => row.get_unwrap(idx),
        }
    }

    pub fn get_ref(&self, idx: &str) -> Result<ValueRef<'_>> {
        match self {
            Row::Prefixed(row) => row.get_ref(idx),
            Row::Standard(row) => row.get_ref(idx),
        }
    }

    pub fn get_ref_unwrap(&self, idx: &str) -> ValueRef<'_> {
        match self {
            Row::Prefixed(row) => row.get_ref_unwrap(idx),
            Row::Standard(row) => row.get_ref_unwrap(idx),
        }
    }

    pub fn with_prefix<T, R>(&self, prefix: &str, callback: T) -> R
    where
        T: FnOnce(&Row) -> R,
    {
        callback(&Row::Prefixed(&PrefixedRow { row: self, prefix }))
    }
}
