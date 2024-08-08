use finnel::prelude::*;

use chrono::{DateTime, Utc};

macro_rules! push_record {
    ( $builder:ident, $($col:expr),* $(,)? ) => {
        {
            use crate::utils::table_display::ColumnDisplay;
            $builder.push_record([$(ColumnDisplay::to_column(&$col),)*])
        }
    }
}

pub trait ColumnDisplay {
    fn to_column(&self) -> String;
}

impl ColumnDisplay for (Amount, Direction) {
    fn to_column(&self) -> String {
        let mut amount = self.0.clone();
        amount.0.set_sign_negative(self.1.is_debit());
        amount.to_string()
    }
}

impl ColumnDisplay for Option<Category> {
    fn to_column(&self) -> String {
        self.as_ref().map(|c| c.name.clone()).to_column()
    }
}

impl ColumnDisplay for (Option<Category>, Option<Category>) {
    fn to_column(&self) -> String {
        if let Some(category) = &self.0 {
            if let Some(parent) = &self.1 {
                format!("{}, {}", category.name, parent.name)
            } else {
                category.name.clone()
            }
        } else {
            String::new()
        }
    }
}

impl ColumnDisplay for Option<Merchant> {
    fn to_column(&self) -> String {
        self.as_ref().map(|c| c.name.clone()).to_column()
    }
}

impl ColumnDisplay for Option<String> {
    fn to_column(&self) -> String {
        self.clone().unwrap_or_else(String::default)
    }
}

impl ColumnDisplay for String {
    fn to_column(&self) -> String {
        self.clone()
    }
}

impl ColumnDisplay for &str {
    fn to_column(&self) -> String {
        self.to_string()
    }
}

impl ColumnDisplay for i64 {
    fn to_column(&self) -> String {
        self.to_string()
    }
}

impl ColumnDisplay for Amount {
    fn to_column(&self) -> String {
        self.to_string()
    }
}

impl ColumnDisplay for Mode {
    fn to_column(&self) -> String {
        self.to_string()
    }
}

impl ColumnDisplay for DateTime<Utc> {
    fn to_column(&self) -> String {
        self.date_naive().to_string()
    }
}
