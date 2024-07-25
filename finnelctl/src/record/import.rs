use std::collections::HashMap;
use std::path::Path;

use finnel::{category::NewCategory, merchant::NewMerchant, prelude::*, record::NewRecord};

use anyhow::Result;
use chrono::{offset::Utc, DateTime, NaiveDate};

mod boursobank;

#[derive(Default)]
pub struct Data {
    pub records: Vec<RecordToImport>,
    merchants: HashMap<String, i64>,
    categories: HashMap<String, i64>,
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
                details,
                mut record,
                merchant_name,
                category_name,
            } in self.records.clone()
            {
                record.account_id = account.id;
                record.details = details.as_str();
                record.merchant_id = self.get_merchant(conn, &merchant_name)?;
                record.category_id = self.get_category(conn, &category_name)?;

                println!("{:#?}", record.save(conn)?);
            }

            Ok(())
        })
    }

    fn get_merchant(&mut self, conn: &mut Conn, name: &str) -> Result<Option<i64>> {
        if name.is_empty() {
            return Ok(None);
        }
        if let Some(id) = self.merchants.get(name) {
            return Ok(Some(*id));
        }

        let merchant = self.find_or_create_merchant(conn, name)?;
        self.merchants.insert(name.to_string(), merchant.id);

        Ok(Some(merchant.id))
    }

    fn find_or_create_merchant(&self, conn: &mut Conn, name: &str) -> Result<Merchant> {
        match Merchant::find_by_name(conn, name) {
            Ok(merchant) => Ok(merchant),
            Err(Error::NotFound) => Ok(NewMerchant::new(name).save(conn)?),
            Err(e) => Err(e.into()),
        }
    }

    fn get_category(&mut self, conn: &mut Conn, name: &str) -> Result<Option<i64>> {
        if name.is_empty() {
            return Ok(None);
        }
        if let Some(id) = self.categories.get(name) {
            return Ok(Some(*id));
        }

        let category = self.find_or_create_category(conn, name)?;
        self.categories.insert(name.to_string(), category.id);

        Ok(Some(category.id))
    }

    fn find_or_create_category(&self, conn: &mut Conn, name: &str) -> Result<Category> {
        match Category::find_by_name(conn, name) {
            Ok(category) => Ok(category),
            Err(Error::NotFound) => Ok(NewCategory::new(name).save(conn)?),
            Err(e) => Err(e.into()),
        }
    }
}

#[derive(Clone)]
pub struct RecordToImport {
    pub details: String,
    pub record: NewRecord<'static>,
    pub merchant_name: String,
    pub category_name: String,
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
