use crate::prelude::*;
use crate::schema::{categories, merchants, records};

use chrono::NaiveDate;

use diesel::{
    expression::SqlLiteral, helper_types::*, prelude::*, sql_types::BigInt, sqlite::Sqlite,
};

diesel::alias! {
    const CATEGORIES: Alias<Categories> = categories as categories_alias;
    const CAT_PARENTS: Alias<CategoriesParent> = categories as categories_parent;
}

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
    pub after: Option<NaiveDate>,
    pub before: Option<NaiveDate>,
    pub operation_date: bool,
    pub greater_than: Option<Decimal>,
    pub less_than: Option<Decimal>,
    pub direction: Option<Direction>,
    pub mode: Option<Mode>,
    pub details: Option<&'a str>,
    pub merchant_id: Option<Option<i64>>,
    pub category_id: Option<Option<i64>>,
    pub category_ids: Option<&'a [i64]>,
    pub count: Option<i64>,
    pub order: Vec<(OrderField, OrderDirection)>,
}

pub struct QueryRecordWithCategory<'a>(QueryRecord<'a>);
pub struct QueryRecordWithCategoryAndParent<'a>(QueryRecord<'a>);
pub struct QueryRecordWithMerchant<'a>(QueryRecord<'a>);
pub struct QueryRecordWithCategoryAndMerchant<'a>(QueryRecord<'a>);
pub struct QueryRecordWithCategoryAndParentAndMerchant<'a>(QueryRecord<'a>);

pub type RecordWithCategory<'a> = (Record, Option<Category>);
pub type RecordWithCategoryAndParent<'a> = (Record, Option<Category>, Option<Category>);
pub type RecordWithMerchant<'a> = (Record, Option<Merchant>);
pub type RecordWithCategoryAndMerchant<'a> = (Record, Option<Category>, Option<Merchant>);
pub type RecordWithCategoryAndParentAndMerchant<'a> =
    (Record, Option<Category>, Option<Category>, Option<Merchant>);

type QueryType<'a> =
    IntoBoxed<'a, Filter<records::table, Eq<records::account_id, SqlLiteral<BigInt>>>, Sqlite>;

impl<'a> QueryRecord<'a> {
    fn sort_by_column<U>(
        query: QueryType<'a>,
        column: U,
        direction: &OrderDirection,
    ) -> QueryType<'a>
    where
        U: 'a
            + ExpressionMethods
            + diesel::query_builder::QueryFragment<Sqlite>
            + AppearsOnTable<records::table>
            + std::marker::Send,
    {
        match direction {
            OrderDirection::Asc => query.then_order_by(column.asc()),
            OrderDirection::Desc => query.then_order_by(column.desc()),
        }
    }

    fn build(&'a self) -> Result<QueryType<'a>> {
        let Some(account_id) = self.account_id else {
            return Err(Error::Invalid("Missing account_id".to_owned()));
        };

        let mut query = records::table
            .into_boxed()
            .filter(records::account_id.eq(account_id));

        if self.operation_date {
            if let Some(date) = self.after {
                query = query.filter(records::operation_date.ge(date));
            }
            if let Some(date) = self.before {
                query = query.filter(records::operation_date.lt(date));
            }
        } else {
            if let Some(date) = self.after {
                query = query.filter(records::value_date.ge(date));
            }
            if let Some(date) = self.before {
                query = query.filter(records::value_date.lt(date));
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
        if let Some(category_ids) = self.category_ids {
            query = query.filter(records::category_id.eq_any(category_ids));
        }
        if let Some(merchant_id) = self.merchant_id {
            query = query.filter(records::merchant_id.is(merchant_id));
        }

        if let Some(count) = self.count {
            query = query.limit(count);
        }

        for (field, direction) in &self.order {
            query = match field {
                OrderField::Amount => Self::sort_by_column(query, records::amount, direction),
                OrderField::Date => {
                    if self.operation_date {
                        Self::sort_by_column(query, records::operation_date, direction)
                    } else {
                        Self::sort_by_column(query, records::value_date, direction)
                    }
                }
                OrderField::CategoryId => {
                    Self::sort_by_column(query, records::category_id, direction)
                }
                OrderField::MerchantId => {
                    Self::sort_by_column(query, records::merchant_id, direction)
                }
            };
        }

        Ok(query)
    }

    pub fn run(&self, conn: &mut Conn) -> Result<Vec<Record>> {
        Ok(self
            .build()?
            .left_join(categories::table)
            .left_join(merchants::table)
            .select(Record::as_select())
            .load::<Record>(conn)?)
    }

    pub fn type_marker(&self) -> std::marker::PhantomData<Record> {
        Default::default()
    }

    pub fn with_category(self) -> QueryRecordWithCategory<'a> {
        QueryRecordWithCategory(self)
    }

    pub fn with_merchant(self) -> QueryRecordWithMerchant<'a> {
        QueryRecordWithMerchant(self)
    }
}

impl<'a> QueryRecordWithCategory<'a> {
    pub fn run(&self, conn: &mut Conn) -> Result<Vec<RecordWithCategory>> {
        Ok(self
            .0
            .build()?
            .left_join(
                CATEGORIES.on(records::category_id.eq(CATEGORIES.field(categories::id).nullable())),
            )
            .select((
                Record::as_select(),
                CATEGORIES.fields(categories::all_columns.nullable()),
            ))
            .load::<RecordWithCategory>(conn)?)
    }

    pub fn type_marker(&self) -> std::marker::PhantomData<RecordWithCategory> {
        Default::default()
    }

    pub fn with_merchant(self) -> QueryRecordWithCategoryAndMerchant<'a> {
        QueryRecordWithCategoryAndMerchant(self.0)
    }

    pub fn with_parent(self) -> QueryRecordWithCategoryAndParent<'a> {
        QueryRecordWithCategoryAndParent(self.0)
    }
}

impl<'a> QueryRecordWithMerchant<'a> {
    pub fn run(&self, conn: &mut Conn) -> Result<Vec<RecordWithMerchant>> {
        Ok(self
            .0
            .build()?
            .left_join(merchants::table)
            .select((Record::as_select(), Option::<Merchant>::as_select()))
            .load::<RecordWithMerchant>(conn)?)
    }

    pub fn type_marker(&self) -> std::marker::PhantomData<RecordWithMerchant> {
        Default::default()
    }

    pub fn with_category(self) -> QueryRecordWithCategoryAndMerchant<'a> {
        QueryRecordWithCategoryAndMerchant(self.0)
    }
}

impl<'a> QueryRecordWithCategoryAndMerchant<'a> {
    pub fn run(&self, conn: &mut Conn) -> Result<Vec<RecordWithCategoryAndMerchant>> {
        Ok(self
            .0
            .build()?
            .left_join(
                CATEGORIES.on(records::category_id.eq(CATEGORIES.field(categories::id).nullable())),
            )
            .left_join(merchants::table)
            .select((
                Record::as_select(),
                CATEGORIES.fields(categories::all_columns.nullable()),
                Option::<Merchant>::as_select(),
            ))
            .load::<RecordWithCategoryAndMerchant>(conn)?)
    }

    pub fn type_marker(&self) -> std::marker::PhantomData<RecordWithCategoryAndMerchant> {
        Default::default()
    }

    pub fn with_parent(self) -> QueryRecordWithCategoryAndParentAndMerchant<'a> {
        QueryRecordWithCategoryAndParentAndMerchant(self.0)
    }
}

impl<'a> QueryRecordWithCategoryAndParent<'a> {
    pub fn run(&self, conn: &mut Conn) -> Result<Vec<RecordWithCategoryAndParent>> {
        Ok(self
            .0
            .build()?
            .left_join(
                CATEGORIES
                    .on(records::category_id.eq(CATEGORIES.field(categories::id).nullable()))
                    .left_join(
                        CAT_PARENTS.on(CATEGORIES
                            .field(categories::parent_id)
                            .eq(CAT_PARENTS.field(categories::id).nullable())),
                    ),
            )
            .select((
                Record::as_select(),
                CATEGORIES.fields(categories::all_columns.nullable()),
                CAT_PARENTS.fields(categories::all_columns.nullable()),
            ))
            .load::<RecordWithCategoryAndParent>(conn)?)
    }

    pub fn type_marker(&self) -> std::marker::PhantomData<RecordWithCategoryAndParent> {
        Default::default()
    }

    pub fn with_merchant(self) -> QueryRecordWithCategoryAndParentAndMerchant<'a> {
        QueryRecordWithCategoryAndParentAndMerchant(self.0)
    }
}

impl<'a> QueryRecordWithCategoryAndParentAndMerchant<'a> {
    pub fn run(&self, conn: &mut Conn) -> Result<Vec<RecordWithCategoryAndParentAndMerchant>> {
        Ok(self
            .0
            .build()?
            .left_join(
                CATEGORIES
                    .on(records::category_id.eq(CATEGORIES.field(categories::id).nullable()))
                    .left_join(
                        CAT_PARENTS.on(CATEGORIES
                            .field(categories::parent_id)
                            .eq(CAT_PARENTS.field(categories::id).nullable())),
                    ),
            )
            .left_join(merchants::table)
            .select((
                Record::as_select(),
                CATEGORIES.fields(categories::all_columns.nullable()),
                CAT_PARENTS.fields(categories::all_columns.nullable()),
                Option::<Merchant>::as_select(),
            ))
            .load::<RecordWithCategoryAndParentAndMerchant>(conn)?)
    }

    pub fn type_marker(&self) -> std::marker::PhantomData<RecordWithCategoryAndParentAndMerchant> {
        Default::default()
    }
}
