use std::marker::PhantomData;

use finnel::{
    prelude::*,
    record::query::{RA, RAC, RACCM, RACM, RC, RCCM, RCM},
};

use chrono::NaiveDate;

macro_rules! table_push_row_elements {
    ( $builder:ident, $($col:expr),* $(,)? ) => {
        {
            use crate::utils::table_display::RowElementDisplay;
            $builder.push_record([$(RowElementDisplay::to_row_element(&$col),)*])
        }
    }
}

macro_rules! table_push_row {
    ( $builder:ident, $row:expr ) => {{
        use crate::utils::table_display::RowDisplay;
        $builder.push_record(RowDisplay::to_row(&$row))
    }};
}

pub fn table_display<T>(rows: Vec<T>)
where
    T: RowDisplay,
    PhantomData<T>: RowDisplay,
{
    if !rows.is_empty() {
        let mut builder = tabled::builder::Builder::new();
        table_push_row!(builder, PhantomData::<T>);
        for result in rows {
            table_push_row!(builder, result);
        }

        println!("{}", builder.build());
    }
}

macro_rules! table_display {
    ( $vec:expr ) => {{
        use crate::utils::table_display::table_display;
        table_display($vec);
    }};
}

pub trait RowDisplay {
    fn to_row(&self) -> Vec<String>;
}

impl RowDisplay for Record {
    fn to_row(&self) -> Vec<String> {
        vec![
            self.id.to_row_element(),
            (self.amount(), self.direction).to_row_element(),
            self.mode.to_row_element(),
            self.operation_date.to_row_element(),
            self.value_date.to_row_element(),
            self.details.to_row_element(),
        ]
    }
}

impl RowDisplay for PhantomData<Record> {
    fn to_row(&self) -> Vec<String> {
        [
            "id",
            "amount",
            "mode",
            "operation date",
            "value date",
            "details",
        ]
        .map(str::to_owned)
        .into_iter()
        .collect()
    }
}

impl RowDisplay for RC {
    fn to_row(&self) -> Vec<String> {
        let mut vec = self.0.to_row();
        vec.extend([self.1.to_row_element()]);
        vec
    }
}

impl RowDisplay for PhantomData<RC> {
    fn to_row(&self) -> Vec<String> {
        let mut vec = PhantomData::<Record>.to_row();
        vec.extend(["category"].map(str::to_owned));
        vec
    }
}

impl RowDisplay for (Record, Option<&Category>, Option<&Merchant>) {
    fn to_row(&self) -> Vec<String> {
        let mut vec = self.0.to_row();
        vec.extend([self.1.to_row_element(), self.2.to_row_element()]);
        vec
    }
}

impl RowDisplay for RCM {
    fn to_row(&self) -> Vec<String> {
        let mut vec = self.0.to_row();
        vec.extend([self.1.to_row_element(), self.2.to_row_element()]);
        vec
    }
}

impl RowDisplay for PhantomData<RCM> {
    fn to_row(&self) -> Vec<String> {
        let mut vec = PhantomData::<Record>.to_row();
        vec.extend(["category", "merchant"].map(str::to_owned));
        vec
    }
}

impl RowDisplay for RCCM {
    fn to_row(&self) -> Vec<String> {
        let mut vec = self.0.to_row();
        vec.extend([
            (self.1.as_ref(), self.2.as_ref()).to_row_element(),
            self.3.to_row_element(),
        ]);
        vec
    }
}

impl RowDisplay for PhantomData<RCCM> {
    fn to_row(&self) -> Vec<String> {
        let mut vec = PhantomData::<Record>.to_row();
        vec.extend(["categories", "merchant"].map(str::to_owned));
        vec
    }
}

impl RowDisplay for RA {
    fn to_row(&self) -> Vec<String> {
        let mut vec = vec![self.1.name.to_row_element()];
        vec.extend(self.0.to_row());
        vec
    }
}

impl RowDisplay for PhantomData<RA> {
    fn to_row(&self) -> Vec<String> {
        let mut vec = vec!["account".to_owned()];
        vec.extend(PhantomData::<Record>.to_row());
        vec
    }
}

impl RowDisplay for RAC {
    fn to_row(&self) -> Vec<String> {
        let mut vec = vec![self.1.name.to_row_element()];
        vec.extend(self.0.to_row());
        vec.extend([self.2.to_row_element()]);
        vec
    }
}

impl RowDisplay for PhantomData<RAC> {
    fn to_row(&self) -> Vec<String> {
        let mut vec = vec!["account".to_owned()];
        vec.extend(PhantomData::<Record>.to_row());
        vec.extend(["category"].map(str::to_owned));
        vec
    }
}

impl RowDisplay for RACM {
    fn to_row(&self) -> Vec<String> {
        let mut vec = vec![self.1.name.to_row_element()];
        vec.extend(self.0.to_row());
        vec.extend([self.2.to_row_element(), self.3.to_row_element()]);
        vec
    }
}

impl RowDisplay for PhantomData<RACM> {
    fn to_row(&self) -> Vec<String> {
        let mut vec = vec!["account".to_owned()];
        vec.extend(PhantomData::<Record>.to_row());
        vec.extend(["category", "merchant"].map(str::to_owned));
        vec
    }
}

impl RowDisplay for RACCM {
    fn to_row(&self) -> Vec<String> {
        let mut vec = vec![self.1.name.to_row_element()];
        vec.extend(self.0.to_row());
        vec.extend([
            (self.2.as_ref(), self.3.as_ref()).to_row_element(),
            self.4.to_row_element(),
        ]);
        vec
    }
}

impl RowDisplay for PhantomData<RACCM> {
    fn to_row(&self) -> Vec<String> {
        let mut vec = PhantomData::<RA>.to_row();
        vec.extend(["categories", "merchant"].map(str::to_owned));
        vec
    }
}

pub trait RowElementDisplay {
    fn to_row_element(&self) -> String;
}

impl RowElementDisplay for (Amount, Direction) {
    fn to_row_element(&self) -> String {
        let mut amount = self.0;
        amount.0.set_sign_negative(self.1.is_debit());
        amount.to_string()
    }
}

impl RowElementDisplay for Category {
    fn to_row_element(&self) -> String {
        self.name.clone()
    }
}

impl RowElementDisplay for Merchant {
    fn to_row_element(&self) -> String {
        self.name.clone()
    }
}

impl RowElementDisplay for Option<Category> {
    fn to_row_element(&self) -> String {
        self.as_ref().map(|c| c.name.clone()).to_row_element()
    }
}

impl RowElementDisplay for Option<&Category> {
    fn to_row_element(&self) -> String {
        self.as_ref().map(|c| c.name.clone()).to_row_element()
    }
}

impl RowElementDisplay for (Option<Category>, Option<Category>) {
    fn to_row_element(&self) -> String {
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

impl RowElementDisplay for (Option<&Category>, Option<&Category>) {
    fn to_row_element(&self) -> String {
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

impl RowElementDisplay for Option<Merchant> {
    fn to_row_element(&self) -> String {
        self.as_ref().map(|c| c.name.clone()).to_row_element()
    }
}

impl RowElementDisplay for Option<&Merchant> {
    fn to_row_element(&self) -> String {
        self.as_ref().map(|c| c.name.clone()).to_row_element()
    }
}

impl RowElementDisplay for Option<String> {
    fn to_row_element(&self) -> String {
        self.clone().unwrap_or_default()
    }
}

impl RowElementDisplay for String {
    fn to_row_element(&self) -> String {
        self.clone()
    }
}

impl RowElementDisplay for &str {
    fn to_row_element(&self) -> String {
        self.to_string()
    }
}

impl RowElementDisplay for i64 {
    fn to_row_element(&self) -> String {
        self.to_string()
    }
}

impl RowElementDisplay for Amount {
    fn to_row_element(&self) -> String {
        self.to_string()
    }
}

impl RowElementDisplay for Mode {
    fn to_row_element(&self) -> String {
        self.to_string()
    }
}

impl RowElementDisplay for NaiveDate {
    fn to_row_element(&self) -> String {
        self.to_string()
    }
}

impl RowElementDisplay for Option<NaiveDate> {
    fn to_row_element(&self) -> String {
        self.map(|d| d.to_row_element()).unwrap_or_default()
    }
}
