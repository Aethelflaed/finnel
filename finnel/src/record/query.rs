use std::marker::PhantomData;

use crate::prelude::*;
use crate::schema::{accounts, categories, merchants, records};

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
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
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

pub type RA = (Record, Account);
pub type RAC = (Record, Account, Option<Category>);
pub type RACM = (Record, Account, Option<Category>, Option<Merchant>);
pub type RACC = (Record, Account, Option<Category>, Option<Category>);
pub type RACCM = (
    Record,
    Account,
    Option<Category>,
    Option<Category>,
    Option<Merchant>,
);
pub type RC = (Record, Option<Category>);
pub type RCC = (Record, Option<Category>, Option<Category>);
pub type RCCM = (Record, Option<Category>, Option<Category>, Option<Merchant>);
pub type RCM = (Record, Option<Category>, Option<Merchant>);

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
        let mut query = records::table.into_boxed();

        if let Some(account_id) = self.account_id {
            query = query.filter(records::account_id.eq(account_id));
        }

        if self.operation_date {
            if let Some(date) = self.from {
                query = query.filter(records::operation_date.ge(date));
            }
            if let Some(date) = self.to {
                query = query.filter(records::operation_date.lt(date));
            }
        } else {
            if let Some(date) = self.from {
                query = query.filter(records::value_date.ge(date));
            }
            if let Some(date) = self.to {
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

    fn load<Q, T>(&self, conn: &mut Conn, query: Q) -> Result<Vec<T>>
    where
        Q: RunQueryDsl<SqliteConnection>
            + diesel::query_dsl::LoadQuery<'a, SqliteConnection, T>
            + diesel::query_builder::QueryFragment<Sqlite>,
    {
        #[cfg(debug_assertions)]
        log::debug!("{:?}", diesel::debug_query::<Sqlite, _>(&query));

        Ok(query.load::<T>(conn)?)
    }

    pub fn run(&self, conn: &mut Conn) -> Result<Vec<Record>> {
        self.load::<_, Record>(conn, self.build()?.select(Record::as_select()))
    }

    pub fn type_marker(&self) -> PhantomData<Record> {
        Default::default()
    }

    pub fn with_account(self) -> BuiltQueryRecord<'a, RA> {
        BuiltQueryRecord::<RA>::build(self)
    }

    pub fn with_category(self) -> BuiltQueryRecord<'a, RC> {
        BuiltQueryRecord::<RC>::build(self)
    }
}

pub struct BuiltQueryRecord<'a, T>(QueryRecord<'a>, PhantomData<T>);

impl<'a, T> BuiltQueryRecord<'a, T> {
    pub fn build(query: QueryRecord<'a>) -> Self {
        BuiltQueryRecord(query, Default::default())
    }

    pub fn load<Q>(&self, conn: &mut Conn, query: Q) -> Result<Vec<T>>
    where
        Q: RunQueryDsl<SqliteConnection>
            + diesel::query_dsl::LoadQuery<'a, SqliteConnection, T>
            + diesel::query_builder::QueryFragment<Sqlite>,
    {
        self.0.load::<_, T>(conn, query)
    }

    pub fn type_marker(&self) -> PhantomData<T> {
        self.1
    }
}

impl<'a> BuiltQueryRecord<'a, RA> {
    pub fn run(&self, conn: &mut Conn) -> Result<Vec<RA>> {
        self.load(
            conn,
            self.0
                .build()?
                .inner_join(accounts::table)
                .select((Record::as_select(), Account::as_select())),
        )
    }

    pub fn with_category(self) -> BuiltQueryRecord<'a, RAC> {
        BuiltQueryRecord::<RAC>::build(self.0)
    }
}

impl<'a> BuiltQueryRecord<'a, RAC> {
    pub fn run(&self, conn: &mut Conn) -> Result<Vec<RAC>> {
        self.load(
            conn,
            self.0
                .build()?
                .inner_join(accounts::table)
                .left_join(
                    CATEGORIES
                        .on(records::category_id.eq(CATEGORIES.field(categories::id).nullable())),
                )
                .select((
                    Record::as_select(),
                    Account::as_select(),
                    CATEGORIES.fields(categories::all_columns.nullable()),
                )),
        )
    }

    pub fn with_merchant(self) -> BuiltQueryRecord<'a, RACM> {
        BuiltQueryRecord::<RACM>::build(self.0)
    }

    pub fn with_parent(self) -> BuiltQueryRecord<'a, RACC> {
        BuiltQueryRecord::<RACC>::build(self.0)
    }
}

impl<'a> BuiltQueryRecord<'a, RACM> {
    pub fn run(&self, conn: &mut Conn) -> Result<Vec<RACM>> {
        self.load(
            conn,
            self.0
                .build()?
                .inner_join(accounts::table)
                .left_join(
                    CATEGORIES
                        .on(records::category_id.eq(CATEGORIES.field(categories::id).nullable())),
                )
                .left_join(merchants::table)
                .select((
                    Record::as_select(),
                    Account::as_select(),
                    CATEGORIES.fields(categories::all_columns.nullable()),
                    Option::<Merchant>::as_select(),
                )),
        )
    }
}

impl<'a> BuiltQueryRecord<'a, RACC> {
    pub fn run(&self, conn: &mut Conn) -> Result<Vec<RACC>> {
        self.load(
            conn,
            self.0
                .build()?
                .inner_join(accounts::table)
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
                    Account::as_select(),
                    CATEGORIES.fields(categories::all_columns.nullable()),
                    CAT_PARENTS.fields(categories::all_columns.nullable()),
                )),
        )
    }

    pub fn with_merchant(self) -> BuiltQueryRecord<'a, RACCM> {
        BuiltQueryRecord::<RACCM>::build(self.0)
    }
}

impl<'a> BuiltQueryRecord<'a, RACCM> {
    pub fn run(&self, conn: &mut Conn) -> Result<Vec<RACCM>> {
        self.load(
            conn,
            self.0
                .build()?
                .inner_join(accounts::table)
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
                    Account::as_select(),
                    CATEGORIES.fields(categories::all_columns.nullable()),
                    CAT_PARENTS.fields(categories::all_columns.nullable()),
                    Option::<Merchant>::as_select(),
                )),
        )
    }
}

impl<'a> BuiltQueryRecord<'a, RC> {
    pub fn run(&self, conn: &mut Conn) -> Result<Vec<RC>> {
        self.load(
            conn,
            self.0
                .build()?
                .left_join(
                    CATEGORIES
                        .on(records::category_id.eq(CATEGORIES.field(categories::id).nullable())),
                )
                .select((
                    Record::as_select(),
                    CATEGORIES.fields(categories::all_columns.nullable()),
                )),
        )
    }

    pub fn with_parent(self) -> BuiltQueryRecord<'a, RCC> {
        BuiltQueryRecord::<RCC>::build(self.0)
    }

    pub fn with_merchant(self) -> BuiltQueryRecord<'a, RCM> {
        BuiltQueryRecord::<RCM>::build(self.0)
    }
}

impl<'a> BuiltQueryRecord<'a, RCC> {
    pub fn run(&self, conn: &mut Conn) -> Result<Vec<RCC>> {
        self.load(
            conn,
            self.0
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
                )),
        )
    }

    pub fn with_merchant(self) -> BuiltQueryRecord<'a, RCCM> {
        BuiltQueryRecord::<RCCM>::build(self.0)
    }
}

impl<'a> BuiltQueryRecord<'a, RCCM> {
    pub fn run(&self, conn: &mut Conn) -> Result<Vec<RCCM>> {
        self.load(
            conn,
            self.0
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
                )),
        )
    }
}

impl<'a> BuiltQueryRecord<'a, RCM> {
    pub fn run(&self, conn: &mut Conn) -> Result<Vec<RCM>> {
        self.load(
            conn,
            self.0
                .build()?
                .left_join(
                    CATEGORIES
                        .on(records::category_id.eq(CATEGORIES.field(categories::id).nullable())),
                )
                .left_join(merchants::table)
                .select((
                    Record::as_select(),
                    CATEGORIES.fields(categories::all_columns.nullable()),
                    Option::<Merchant>::as_select(),
                )),
        )
    }
}
