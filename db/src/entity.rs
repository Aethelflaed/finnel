use super::{Connection, Id, Result, Row};

pub trait Entity: for<'a> TryFrom<&'a Row<'a>> + Sized {
    fn id(&self) -> Option<Id>;

    fn find(db: &Connection, id: Id) -> Result<Self>;
    fn save(&mut self, db: &Connection) -> Result<()>;
}

pub trait EntityDescriptor {
    fn table_name() -> &'static str;
    fn field_names() -> &'static [&'static str];
}
