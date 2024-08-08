macro_rules! push_record {
    ( $builder:ident, $($col:expr),* $(,)? ) => {
        {
            use crate::utils::table_display::TableDisplay;
            $builder.push_record([$(TableDisplay::to_column(&$col),)*])
        }
    }
}

pub trait TableDisplay {
    fn to_column(&self) -> String;
}

impl TableDisplay for Option<String> {
    fn to_column(&self) -> String {
        self.clone().unwrap_or_else(String::default)
    }
}

impl TableDisplay for String {
    fn to_column(&self) -> String {
        self.clone()
    }
}

impl TableDisplay for &str {
    fn to_column(&self) -> String {
        self.to_string()
    }
}

impl TableDisplay for i64 {
    fn to_column(&self) -> String {
        self.to_string()
    }
}

impl TableDisplay for finnel::Amount {
    fn to_column(&self) -> String {
        self.to_string()
    }
}
