use crate::prelude::*;
use crate::schema::records;

use chrono::{offset::Utc, DateTime};

use diesel::{
    expression::SqlLiteral, helper_types::*, prelude::*, sql_types::BigInt, sqlite::Sqlite,
};

#[derive(Debug, Clone, Copy)]
pub enum OrderField {
    Amount,
    Date,
    CategoryId,
    MerchantId,
}

#[derive(Debug, Clone, Copy)]
pub enum OrderDirection {
    Asc,
    Desc,
}

#[derive(Default)]
pub struct QueryRecord<'a> {
    pub account_id: Option<i64>,
    pub after: Option<DateTime<Utc>>,
    pub before: Option<DateTime<Utc>>,
    pub operation_date: bool,
    pub greater_than: Option<Decimal>,
    pub less_than: Option<Decimal>,
    pub direction: Option<Direction>,
    pub mode: Option<Mode>,
    pub details: Option<&'a str>,
    pub merchant_id: Option<Option<i64>>,
    pub category_id: Option<Option<i64>>,
    pub count: Option<i64>,
    pub order: Vec<(OrderField, OrderDirection)>,
}

type QueryRecordResult = (Record, Option<Category>, Option<Merchant>);

type QueryType<'a> =
    IntoBoxed<'a, Filter<records::table, Eq<records::account_id, SqlLiteral<BigInt>>>, Sqlite>;

impl QueryRecord<'_> {
    fn sort_by_column<'a, U>(
        query: QueryType<'a>,
        column: U,
        direction: &OrderDirection,
        first: bool,
    ) -> QueryType<'a>
    where
        U: 'a
            + ExpressionMethods
            + diesel::query_builder::QueryFragment<Sqlite>
            + AppearsOnTable<records::table>
            + std::marker::Send,
    {
        if first {
            match direction {
                OrderDirection::Asc => query.order_by(column.asc()),
                OrderDirection::Desc => query.order_by(column.desc()),
            }
        } else {
            match direction {
                OrderDirection::Asc => query.then_order_by(column.asc()),
                OrderDirection::Desc => query.then_order_by(column.desc()),
            }
        }
    }

    pub fn run(&self, conn: &mut Conn) -> Result<Vec<QueryRecordResult>> {
        let Some(account_id) = self.account_id else {
            return Err(Error::Invalid("Missing account_id".to_owned()));
        };

        let mut query = records::table
            .into_boxed()
            .filter(records::account_id.eq(account_id));

        if self.operation_date {
            if let Some(date) = self.after {
                query = query.filter(records::operation_date.lt(date));
            }
            if let Some(date) = self.before {
                query = query.filter(records::operation_date.ge(date));
            }
        } else {
            if let Some(date) = self.after {
                query = query.filter(records::value_date.lt(date));
            }
            if let Some(date) = self.before {
                query = query.filter(records::value_date.ge(date));
            }
        }

        if let Some(amount) = self.greater_than {
            query = query.filter(records::amount.ge(crate::db::Decimal(amount)));
        }
        if let Some(amount) = self.less_than {
            query = query.filter(records::amount.lt(crate::db::Decimal(amount)));
        }
        if let Some(direction) = self.direction {
            query = query.filter(records::direction.eq(direction));
        }
        if let Some(mode) = &self.mode {
            query = query.filter(records::mode.eq(mode));
        }
        if let Some(details) = self.details {
            query = query.filter(records::details.like(details));
        }
        if let Some(category_id) = self.category_id {
            query = query.filter(records::category_id.is(category_id));
        }
        if let Some(merchant_id) = self.merchant_id {
            query = query.filter(records::merchant_id.is(merchant_id));
        }

        if let Some(count) = self.count {
            query = query.limit(count);
        }

        let mut first_order = true;
        for (field, direction) in &self.order {
            query = match field {
                OrderField::Amount => {
                    Self::sort_by_column(query, records::amount, direction, first_order)
                }
                OrderField::Date => {
                    if self.operation_date {
                        Self::sort_by_column(query, records::operation_date, direction, first_order)
                    } else {
                        Self::sort_by_column(query, records::value_date, direction, first_order)
                    }
                }
                OrderField::CategoryId => {
                    Self::sort_by_column(query, records::category_id, direction, first_order)
                }
                OrderField::MerchantId => {
                    Self::sort_by_column(query, records::merchant_id, direction, first_order)
                }
            };

            first_order = false;
        }

        Ok(query
            .left_join(crate::schema::categories::table)
            .left_join(crate::schema::merchants::table)
            .select((
                Record::as_select(),
                Option::<Category>::as_select(),
                Option::<Merchant>::as_select(),
            ))
            .load::<QueryRecordResult>(conn)?)
    }
}
