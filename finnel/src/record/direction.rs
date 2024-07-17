use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

use crate::result::ParseTypeError;

use diesel::{
    backend::Backend,
    deserialize::{self, FromSql, FromSqlRow},
    expression::AsExpression,
    serialize::{self, IsNull, Output, ToSql},
    sql_types::Text,
    sqlite::Sqlite,
};

#[derive(
    Default, Debug, Clone, Copy, PartialEq, Eq, FromSqlRow, AsExpression,
)]
#[diesel(sql_type = Text)]
pub enum Direction {
    #[default]
    Debit,
    Credit,
}

impl Direction {
    pub fn is_debit(&self) -> bool {
        self == &Direction::Debit
    }
    pub fn is_credit(&self) -> bool {
        self == &Direction::Credit
    }
}

use Direction::*;

impl Display for Direction {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Debit => f.write_str("Debit"),
            Credit => f.write_str("Credit"),
        }
    }
}

impl FromStr for Direction {
    type Err = ParseTypeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_lowercase().as_str() {
            "debit" => Ok(Debit),
            "débit" => Ok(Debit),
            "credit" => Ok(Credit),
            "crédit" => Ok(Credit),
            _ => Err(ParseTypeError("Direction", value.to_string())),
        }
    }
}

impl ToSql<Text, Sqlite> for Direction {
    fn to_sql<'b>(
        &'b self,
        out: &mut Output<'b, '_, Sqlite>,
    ) -> serialize::Result {
        out.set_value(self.to_string());
        Ok(IsNull::No)
    }
}

impl FromSql<Text, Sqlite> for Direction {
    fn from_sql(
        bytes: <Sqlite as Backend>::RawValue<'_>,
    ) -> deserialize::Result<Self> {
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
        assert_eq!(Debit, "Debit".parse::<Direction>()?);
        assert_eq!(Debit, "Débit".parse::<Direction>()?);
        assert_eq!(Debit, "débit".parse::<Direction>()?);
        assert_eq!(Credit, "Credit".parse::<Direction>()?);
        assert_eq!(Credit, "Crédit".parse::<Direction>()?);
        assert_eq!(Credit, "crédit".parse::<Direction>()?);

        assert!(Debit.to_string().parse::<Direction>().is_ok());
        assert!(Credit.to_string().parse::<Direction>().is_ok());

        Ok(())
    }
}
