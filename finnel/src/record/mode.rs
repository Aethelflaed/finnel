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

#[derive(Debug, Clone, Copy, PartialEq, Eq, FromSqlRow, AsExpression)]
#[diesel(sql_type = Text)]
pub enum Mode {
    Direct(PaymentMethod),
    Atm(PaymentMethod),
    Transfer,
}

use Mode::*;

impl Default for Mode {
    fn default() -> Self {
        Direct(Empty)
    }
}

impl Display for Mode {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Atm(Empty) => f.write_str("ATM"),
            Atm(method) => f.write_fmt(format_args!("ATM {}", method)),
            Direct(Empty) => f.write_str("Direct"),
            Direct(method) => method.fmt(f),
            Transfer => f.write_str("Transfer"),
        }
    }
}

impl FromStr for Mode {
    type Err = ParseTypeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_lowercase().as_str() {
            "direct" => Ok(Direct(Empty)),
            card if PaymentMethod::guard(card, "card") => {
                PaymentMethod::read(card, "card").map(Direct)
            }
            "atm" => Ok(Atm(Empty)),
            atm if PaymentMethod::guard(atm, "atm card") => {
                PaymentMethod::read(atm, "atm card").map(Atm)
            }
            "transfer" => Ok(Transfer),
            _ => Err(ParseTypeError("Mode", value.to_string())),
        }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaymentMethod {
    Empty,
    CardLast4Digit(char, char, char, char),
}

use PaymentMethod::*;

impl PaymentMethod {
    pub fn guard(value: &str, prefix: &str) -> bool {
        value.is_empty() || Self::last_4_guard(value, prefix)
    }

    pub fn read(value: &str, prefix: &str) -> Result<Self, ParseTypeError> {
        Self::last_4_read(value, prefix)
    }

    fn last_4_guard(value: &str, prefix: &str) -> bool {
        let mut chars = value.chars().skip(prefix.len());

        value.starts_with(prefix)
            && value.len() == prefix.len() + 6
            && chars.next() == Some(' ')
            && chars.next() == Some('*')
            && chars.all(|c| c.is_ascii_digit())
    }

    fn last_4_read(value: &str, prefix: &str) -> Result<Self, ParseTypeError> {
        let mut chars = value.chars().skip(prefix.len());

        if let (Some(' '), Some('*'), Some(a), Some(b), Some(c), Some(d)) = (
            chars.next(),
            chars.next(),
            chars.next(),
            chars.next(),
            chars.next(),
            chars.next(),
        ) {
            Ok(CardLast4Digit(a, b, c, d))
        } else {
            Err(ParseTypeError("PaymentMethod", value.to_owned()))
        }
    }
}

impl Display for PaymentMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        match self {
            Empty => Ok(()),
            CardLast4Digit(a, b, c, d) => {
                f.write_fmt(format_args!("Card *{a}{b}{c}{d}"))
            }
        }
    }
}

impl FromStr for PaymentMethod {
    type Err = ParseTypeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_lowercase().as_str() {
            "" => Ok(Empty),
            card if Self::last_4_guard(card, "card") => {
                Self::last_4_read(card, "card")
            }
            _ => Err(ParseTypeError("PaymentMethod", value.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use pretty_assertions::assert_eq;

    macro_rules! last_4 {
        ($($c:literal),+) => {
            CardLast4Digit($(char::from_digit($c as u32, 10).unwrap()),+)
        }
    }

    #[test]
    fn direct() -> Result<()> {
        assert_eq!(Direct(Empty), "direct".parse::<Mode>()?);
        assert_eq!(Direct(Empty), "Direct".parse::<Mode>()?);
        assert_eq!(Direct(last_4!(1, 4, 2, 3)), "card *1423".parse::<Mode>()?);

        assert!(Direct(Empty).to_string().parse::<Mode>().is_ok());
        assert!(Direct(last_4!(1, 4, 2, 3))
            .to_string()
            .parse::<Mode>()
            .is_ok());

        Ok(())
    }

    #[test]
    fn atm() -> Result<()> {
        assert_eq!(Atm(Empty), "ATM".parse::<Mode>()?);
        assert_eq!(Atm(last_4!(1, 2, 3, 4)), "ATM Card *1234".parse::<Mode>()?);

        assert!(Atm(Empty).to_string().parse::<Mode>().is_ok());
        assert!(Atm(last_4!(1, 4, 2, 3)).to_string().parse::<Mode>().is_ok());

        Ok(())
    }

    #[test]
    fn transfer() -> Result<()> {
        assert_eq!(Transfer, "transfer".parse::<Mode>()?);
        assert!(Transfer.to_string().parse::<Mode>().is_ok());

        Ok(())
    }
}
