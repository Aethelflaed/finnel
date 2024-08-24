use crate::prelude::*;

mod categories;
mod merchants;
mod records;
mod reports;

pub fn consolidate(conn: &mut Conn) -> Result<()> {
    categories::consolidate(conn)?;
    merchants::consolidate(conn)?;
    records::consolidate(conn)?;
    reports::consolidate(conn)?;

    Ok(())
}
