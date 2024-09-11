use diesel::{
    backend::Backend,
    deserialize::{self, FromSql, FromSqlRow},
    expression::AsExpression,
    serialize::{self, IsNull, Output, ToSql},
    sql_types::Text,
    sqlite::Sqlite,
};
use crate::result::ParseTypeError;
use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Text)]
pub enum Frequency {
    Weekly,
    #[default]
    Monthly,
}

use Frequency::*;

impl Display for Frequency {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Weekly => f.write_str("Weekly"),
            Monthly => f.write_str("Monthly"),
        }
    }
}

impl FromStr for Frequency {
    type Err = ParseTypeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_lowercase().as_str() {
            "weekly" => Ok(Weekly),
            "monthly" => Ok(Monthly),
            _ => Err(ParseTypeError("Frequency", value.to_string())),
        }
    }
}

impl ToSql<Text, Sqlite> for Frequency {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        out.set_value(self.to_string());
        Ok(IsNull::No)
    }
}

impl FromSql<Text, Sqlite> for Frequency {
    fn from_sql(bytes: <Sqlite as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        Ok(<String as FromSql<Text, Sqlite>>::from_sql(bytes)?.parse()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use pretty_assertions::assert_eq;

    #[test]
    fn from_str() -> Result<()> {
        assert_eq!(Weekly, "Weekly".parse::<Frequency>()?);
        assert_eq!(Weekly, "weekly".parse::<Frequency>()?);
        assert_eq!(Monthly, "Monthly".parse::<Frequency>()?);
        assert_eq!(Monthly, "monthly".parse::<Frequency>()?);

        Ok(())
    }

    #[test]
    fn display_and_from_str_interoperability() -> Result<()> {
        assert!(Weekly.to_string().parse::<Frequency>().is_ok());
        assert!(Monthly.to_string().parse::<Frequency>().is_ok());

        Ok(())
    }
}
