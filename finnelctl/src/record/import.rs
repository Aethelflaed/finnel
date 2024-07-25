use std::collections::HashMap;
use std::path::Path;

use finnel::{category::NewCategory, merchant::NewMerchant, prelude::*, record::NewRecord};

use anyhow::Result;
use chrono::{offset::Utc, DateTime, NaiveDate};

mod boursobank;

#[derive(Default)]
pub struct Data {
    pub records: Vec<RecordToImport>,
    categories: HashMap<String, Category>,
    merchants: HashMap<String, Merchant>,
}

impl Data {
    pub fn new(records: Vec<RecordToImport>) -> Self {
        Data {
            records,
            ..Default::default()
        }
    }

    pub fn persist(&mut self, account: &Account, conn: &mut Conn) -> Result<()> {
        conn.transaction(|conn| {
            for RecordToImport {
                amount,
                operation_date,
                value_date,
                direction,
                mode,
                details,
                category_name,
                merchant_name,
            } in self.records.clone()
            {
                self.add_category(conn, &category_name)?;
                self.add_merchant(conn, &merchant_name)?;

                let record = NewRecord {
                    amount,
                    operation_date,
                    value_date,
                    direction,
                    mode,
                    details: details.as_str(),
                    category: self.get_category(&category_name),
                    merchant: self.get_merchant(&merchant_name),
                    ..NewRecord::new(account)
                };

                println!("{:#?}", record.save(conn)?);
            }

            Ok(())
        })
    }

    fn get_category(&self, name: &str) -> Option<&Category> {
        if name.is_empty() {
            None
        } else {
            self.categories.get(name)
        }
    }

    fn add_category(&mut self, conn: &mut Conn, name: &str) -> Result<()> {
        if !name.is_empty() && !self.categories.contains_key(name) {
            self.categories.insert(name.to_string(), self.find_or_create_category(conn, name)?);
        }

        Ok(())
    }

    fn find_or_create_category(&self, conn: &mut Conn, name: &str) -> Result<Category> {
        match Category::find_by_name(conn, name) {
            Ok(category) => Ok(category),
            Err(Error::NotFound) => Ok(NewCategory::new(name).save(conn)?),
            Err(e) => Err(e.into()),
        }
    }


    fn get_merchant(&self, name: &str) -> Option<&Merchant> {
        if name.is_empty() {
            None
        } else {
            self.merchants.get(name)
        }
    }

    fn add_merchant(&mut self, conn: &mut Conn, name: &str) -> Result<()> {
        if !name.is_empty() && !self.merchants.contains_key(name) {
            self.merchants.insert(name.to_string(), self.find_or_create_merchant(conn, name)?);
        }

        Ok(())
    }

    fn find_or_create_merchant(&self, conn: &mut Conn, name: &str) -> Result<Merchant> {
        match Merchant::find_by_name(conn, name) {
            Ok(merchant) => Ok(merchant),
            Err(Error::NotFound) => Ok(NewMerchant::new(name).save(conn)?),
            Err(e) => Err(e.into()),
        }
    }
}

#[derive(Default, Clone)]
pub struct RecordToImport {
    pub operation_date: DateTime<Utc>,
    pub value_date: DateTime<Utc>,
    pub amount: Decimal,
    pub direction: Direction,
    pub mode: Mode,
    pub details: String,
    pub category_name: String,
    pub merchant_name: String,
}

pub fn import<T: AsRef<Path>, S: AsRef<str>>(profile: S, path: T) -> Result<Data> {
    match profile.as_ref().to_lowercase().as_str() {
        "boursobank" => Ok(boursobank::Importer::import(path)?),
        _ => Err(anyhow::anyhow!("Unknown profile {}", profile.as_ref())),
    }
}

trait Profile {
    fn import<T: AsRef<Path>>(path: T) -> Result<Data>;
}

fn parse_date_fmt(date: &str, fmt: &str) -> Result<DateTime<Utc>> {
    crate::utils::naive_date_to_utc(NaiveDate::parse_from_str(date, fmt)?)
}
