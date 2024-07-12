use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

use diesel::{
    backend::Backend,
    deserialize::{self, FromSql, FromSqlRow},
    expression::AsExpression,
    serialize::{self, IsNull, Output, ToSql},
    sql_types::Text,
    sqlite::Sqlite,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq,
    FromSqlRow, AsExpression)]
#[diesel(sql_type = Text)]
pub enum Direction {
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

#[derive(Debug, Clone, PartialEq, Eq,
    FromSqlRow, AsExpression)]
#[diesel(sql_type = Text)]
pub enum Mode {
    Direct,
    Transfer,
    Atm,
    Other(String),
}

use Direction::*;
use Mode::*;

impl Display for Direction {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Debit => f.write_str("Debit"),
            Credit => f.write_str("Credit"),
        }
    }
}
impl Display for Mode {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Transfer => f.write_str("Transfer"),
            Atm => f.write_str("ATM"),
            Direct => f.write_str("Direct"),
            Other(string) => f.write_str(string.as_str()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub struct ParseTypeError;

impl Display for ParseTypeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "Parse Type Error")
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
            _ => Err(ParseTypeError),
        }
    }
}

impl FromStr for Mode {
    type Err = ParseTypeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_lowercase().as_str() {
            "direct" => Ok(Direct),
            "transfer" => Ok(Transfer),
            "atm" => Ok(Atm),
            "" => Err(ParseTypeError),
            other if other.chars().all(char::is_whitespace) => {
                Err(ParseTypeError)
            }
            _ => Ok(Other(value.to_string())),
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

impl ToSql<Text, Sqlite> for Mode {
    fn to_sql<'b>(
        &'b self,
        out: &mut Output<'b, '_, Sqlite>,
    ) -> serialize::Result {
        out.set_value(self.to_string());
        Ok(IsNull::No)
    }
}

impl FromSql<Text, Sqlite> for Mode {
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

        assert_eq!(Direct, "direct".parse::<Mode>()?);
        assert_eq!(Direct, "Direct".parse::<Mode>()?);
        assert_eq!(Transfer, "transfer".parse::<Mode>()?);
        assert_eq!(Atm, "ATM".parse::<Mode>()?);
        assert_eq!(Other("foo".to_string()), "foo".parse::<Mode>()?);

        Ok(())
    }
}
