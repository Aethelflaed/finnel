use std::collections::HashMap;
use std::path::Path;

use finnel::{
    record::NewRecord, Account, Category, Connection, Entity, Error, Id,
    Merchant,
};

use anyhow::{Context, Result};
use chrono::{offset::Utc, DateTime, NaiveDate};

mod boursobank;

#[derive(Default, Debug)]
pub struct Data {
    pub records: Vec<RecordToImport>,
    merchants: HashMap<String, Id>,
    categories: HashMap<String, Id>,
}

impl Data {
    pub fn new(records: Vec<RecordToImport>) -> Self {
        Data {
            records,
            ..Default::default()
        }
    }

    pub fn persist(
        &mut self,
        account: &Account,
        db: &mut Connection,
    ) -> Result<()> {
        let tx = db.transaction()?;

        for RecordToImport {
            mut record,
            merchant_name,
            category_name,
            payment_method: _,
        } in self.records.clone()
        {
            record.account_id = account.id();
            record.merchant_id = self.get_merchant(&tx, merchant_name)?;
            record.category_id = self.get_category(&tx, category_name)?;

            println!("{:#?}", record.save(&tx)?);
        }

        tx.commit()?;

        Ok(())
    }

    fn get_merchant(
        &mut self,
        db: &Connection,
        name: String,
    ) -> Result<Option<Id>> {
        if name.is_empty() {
            return Ok(None);
        }
        if let Some(id) = self.merchants.get(&name) {
            return Ok(Some(*id));
        }

        let merchant = self.find_or_create_merchant(db, name.clone())?;
        let id = merchant.id().context("Merchant id absent")?;
        self.merchants.insert(name.clone(), id);

        Ok(Some(id))
    }

    fn find_or_create_merchant(
        &self,
        db: &Connection,
        name: String,
    ) -> Result<Merchant> {
        match Merchant::find_by_name(db, name.as_str()) {
            Ok(merchant) => Ok(merchant),
            Err(Error::NotFound) => {
                let mut merchant = Merchant::new(name);
                merchant.save(db)?;
                Ok(merchant)
            }
            Err(e) => Err(e.into()),
        }
    }

    fn get_category(
        &mut self,
        db: &Connection,
        name: String,
    ) -> Result<Option<Id>> {
        if name.is_empty() {
            return Ok(None);
        }
        if let Some(id) = self.categories.get(&name) {
            return Ok(Some(*id));
        }

        let category = self.find_or_create_category(db, name.clone())?;
        let id = category.id().context("Category id absent")?;
        self.categories.insert(name.clone(), id);

        Ok(Some(id))
    }

    fn find_or_create_category(
        &self,
        db: &Connection,
        name: String,
    ) -> Result<Category> {
        match Category::find_by_name(db, name.as_str()) {
            Ok(category) => Ok(category),
            Err(Error::NotFound) => {
                let mut category = Category::new(name);
                category.save(db)?;
                Ok(category)
            }
            Err(e) => Err(e.into()),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct RecordToImport {
    pub record: NewRecord,
    pub merchant_name: String,
    pub category_name: String,
    pub payment_method: String,
}

pub fn import<T: AsRef<Path>, S: AsRef<str>>(
    profile: S,
    path: T,
) -> Result<Data> {
    match profile.as_ref().to_lowercase().as_str() {
        "boursobank" => Ok(boursobank::Importer::import(path)?),
        _ => Err(anyhow::anyhow!("Unknown profile {}", profile.as_ref())),
    }
}

trait Profile {
    fn import<T: AsRef<Path>>(path: T) -> Result<Data>;
}

fn parse_date_fmt(date: &str, fmt: &str) -> Result<DateTime<Utc>> {
    crate::cli::naive_date_to_utc(NaiveDate::parse_from_str(date, fmt)?)
}
