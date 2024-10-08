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

    monthly_category_stats (id) {
        id -> BigInt,
        year -> Integer,
        month -> Integer,
        amount -> BigInt,
        currency -> Text,
        category_id -> Nullable<BigInt>,
        direction -> Text,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    monthly_stats (year, month, currency) {
        year -> Integer,
        month -> Integer,
        debit_amount -> BigInt,
        credit_amount -> BigInt,
        currency -> Text,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    records (id) {
        id -> BigInt,
        account_id -> BigInt,
        amount -> BigInt,
        currency -> Text,
        operation_date -> Date,
        value_date -> Date,
        direction -> Text,
        mode -> Text,
        details -> Text,
        category_id -> Nullable<BigInt>,
        merchant_id -> Nullable<BigInt>,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    recurring_payments (id) {
        id -> BigInt,
        name -> Text,
        description -> Text,
        frequency -> Text,
        account_id -> BigInt,
        amount -> BigInt,
        currency -> Text,
        direction -> Text,
        mode -> Text,
        category_id -> Nullable<BigInt>,
        merchant_id -> Nullable<BigInt>,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    reports (id) {
        id -> BigInt,
        name -> Text,
    }
}

diesel::table! {
    use diesel::sql_types::*;

    reports_categories (report_id, category_id) {
        report_id -> BigInt,
        category_id -> BigInt,
    }
}

diesel::joinable!(merchants -> categories (default_category_id));
diesel::joinable!(monthly_category_stats -> categories (category_id));
diesel::joinable!(records -> accounts (account_id));
diesel::joinable!(records -> categories (category_id));
diesel::joinable!(records -> merchants (merchant_id));
diesel::joinable!(recurring_payments -> accounts (account_id));
diesel::joinable!(recurring_payments -> categories (category_id));
diesel::joinable!(recurring_payments -> merchants (merchant_id));
diesel::joinable!(reports_categories -> categories (category_id));
diesel::joinable!(reports_categories -> reports (report_id));

diesel::allow_tables_to_appear_in_same_query!(
    accounts,
    categories,
    merchants,
    monthly_category_stats,
    monthly_stats,
    records,
    recurring_payments,
    reports,
    reports_categories,
);
