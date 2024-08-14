use oxydized_money::CurrencyError;

use diesel::{
    prelude::*,
    backend::Backend,
    deserialize::{self, FromSql, FromSqlRow},
    expression::AsExpression,
    serialize::{self, IsNull, Output, ToSql},
    sql_types::{BigInt, Text},
    sqlite::Sqlite,
};

define_sql_function! {
    /// Like sum, but returns 0 instead of NULL
    ///
    /// Additionally, the type constraint makes sum (and total) return an integer instead of a
    /// double
    #[aggregate]
    #[sql_name = "TOTAL"]
    fn total(x: BigInt) -> BigInt;
}

#[derive(Copy, Clone, Debug, derive_more::From, derive_more::Into, FromSqlRow, AsExpression)]
#[diesel(sql_type = BigInt)]
pub struct Decimal(pub oxydized_money::Decimal);

impl ToSql<BigInt, Sqlite> for Decimal {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        let mut value = self.0;
        value.rescale(3);

        match TryInto::<i64>::try_into(value.mantissa()) {
            Ok(value) => {
                out.set_value(value);
                Ok(IsNull::No)
            }
            Err(e) => Err(Box::new(e)),
        }
    }
}

impl FromSql<BigInt, Sqlite> for Decimal {
    fn from_sql(bytes: <Sqlite as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        Ok(oxydized_money::Decimal::new(i64::from_sql(bytes)?, 3).into())
    }
}

#[derive(Copy, Clone, Debug, derive_more::From, derive_more::Into, FromSqlRow, AsExpression)]
#[diesel(sql_type = Text)]
pub struct Currency(pub oxydized_money::Currency);

impl ToSql<Text, Sqlite> for Currency {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Sqlite>) -> serialize::Result {
        <str as ToSql<Text, Sqlite>>::to_sql(self.0.code(), out)
    }
}

impl FromSql<Text, Sqlite> for Currency {
    fn from_sql(bytes: <Sqlite as Backend>::RawValue<'_>) -> deserialize::Result<Self> {
        match oxydized_money::Currency::from_code(
            <String as FromSql<Text, Sqlite>>::from_sql(bytes)?.as_str(),
        ) {
            Some(cur) => Ok(Currency(cur)),
            None => Err(Box::new(CurrencyError::Unknown)),
        }
    }
}
