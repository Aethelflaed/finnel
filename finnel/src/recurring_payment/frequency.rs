use diesel::{
    backend::Backend,
    deserialize::{self, FromSql, FromSqlRow},
    expression::AsExpression,
    serialize::{self, IsNull, Output, ToSql},
    sql_types::Text,
    sqlite::Sqlite,
};
use derive_more::{Display, FromStr};

#[derive(Default, Debug, Display, Clone, Copy, PartialEq, Eq, FromSqlRow, AsExpression, FromStr)]
#[diesel(sql_type = Text)]
pub enum Frequency {
    Weekly,
    #[default]
    Monthly,
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
