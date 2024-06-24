use std::fmt::{Display, Error, Formatter};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, derive_more::From)]
pub struct Type(Direction, Mode);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Debit,
    Credit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Direct,
    Transfer,
    Atm,
    Other(String),
}

use Direction::*;
use Mode::*;

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        if let Direct = self.1 {
            write!(f, "{}", self.0)
        } else {
            write!(f, "{} {}", self.0, self.1)
        }
    }
}

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
            Transfer => f.write_str("transfer"),
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
            other if other.is_empty() => Err(ParseTypeError),
            _ => Ok(Other(value.to_string())),
        }
    }
}

impl FromStr for Type {
    type Err = ParseTypeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut words = value.split_whitespace();

        let direction =
            words.next().ok_or(ParseTypeError)?.parse::<Direction>()?;

        if let Some(mode) = words.next() {
            let None = words.next() else {
                return Err(ParseTypeError);
            };

            Ok((direction, mode.parse::<Mode>()?).into())
        } else {
            Ok((direction, Direct).into())
        }
    }
}

use rusqlite::types::{FromSql, FromSqlResult, ToSql, ToSqlOutput, ValueRef};

impl FromSql for Direction {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        Self::from_str(value.as_str()?)
            .map_err(|e| rusqlite::types::FromSqlError::Other(Box::new(e)))
    }
}

impl ToSql for Direction {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Owned(rusqlite::types::Value::Text(
            self.to_string(),
        )))
    }
}

impl FromSql for Mode {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        Self::from_str(value.as_str()?)
            .map_err(|e| rusqlite::types::FromSqlError::Other(Box::new(e)))
    }
}

impl ToSql for Mode {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Owned(rusqlite::types::Value::Text(
            self.to_string(),
        )))
    }
}

impl FromSql for Type {
    fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
        Self::from_str(value.as_str()?)
            .map_err(|e| rusqlite::types::FromSqlError::Other(Box::new(e)))
    }
}

impl ToSql for Type {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Owned(rusqlite::types::Value::Text(
            self.to_string(),
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use pretty_assertions::assert_eq;

    #[test]
    fn from_str() -> Result<()> {
        assert_eq!(Type(Debit, Direct), Type::from_str("Debit")?);
        assert_eq!(Type(Credit, Direct), Type::from_str("Credit")?);
        assert_eq!(Type(Debit, Transfer), Type::from_str("Debit Transfer")?);
        assert_eq!(Type(Credit, Transfer), Type::from_str("Credit Transfer")?);
        assert_eq!(Type(Debit, Atm), Type::from_str("Debit ATM")?);
        assert_eq!(Type(Credit, Atm), Type::from_str("Credit ATM")?);
        assert_eq!(
            Type(Debit, Other("foo".to_string())),
            Type::from_str("Debit foo")?
        );
        assert_eq!(
            Type(Credit, Other("foo".to_string())),
            Type::from_str("Credit foo")?
        );

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

        Ok(())
    }
}
