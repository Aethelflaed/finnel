use crate::prelude::*;

mod reports;

pub fn consolidate(conn: &mut Conn) -> Result<()> {
    reports::consolidate(conn)?;

    Ok(())
}
