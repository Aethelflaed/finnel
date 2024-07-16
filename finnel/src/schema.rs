// @generated automatically by Diesel CLI.

diesel::table! {
    use diesel::sql_types::*;

    accounts (id) {
        id -> BigInt,
        name -> Text,
        balance -> BigInt,
        currency -> Text,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    categories (id) {
        id -> BigInt,
        name -> Text,
        parent_id -> Nullable<BigInt>,
        replaced_by_id -> Nullable<BigInt>,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    merchants (id) {
        id -> BigInt,
        name -> Text,
        default_category_id -> Nullable<BigInt>,
        replaced_by_id -> Nullable<BigInt>,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    records (id) {
        id -> BigInt,
        account_id -> BigInt,
        amount -> BigInt,
        currency -> Text,
        operation_date -> TimestamptzSqlite,
        value_date -> TimestamptzSqlite,
        direction -> Text,
        mode -> Text,
        details -> Text,
        category_id -> Nullable<BigInt>,
        merchant_id -> Nullable<BigInt>,
    }
}

diesel::joinable!(merchants -> categories (default_category_id));
diesel::joinable!(records -> accounts (account_id));
diesel::joinable!(records -> categories (category_id));
diesel::joinable!(records -> merchants (merchant_id));

diesel::allow_tables_to_appear_in_same_query!(
    accounts, categories, merchants, records,
);