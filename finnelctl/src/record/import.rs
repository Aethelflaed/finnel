use std::path::Path;

use finnel::record::NewRecord;

use anyhow::Result;
use chrono::{offset::Utc, DateTime, NaiveDate};

mod boursobank;

#[derive(Debug, Default)]
pub struct Data {
    pub record: NewRecord,
    pub merchant_name: String,
    pub category_name: String,
    pub payment_method: String,
}

pub fn import<T: AsRef<Path>, S: AsRef<str>>(
    profile: S,
    path: T,
) -> Result<Vec<Data>> {
    match profile.as_ref().to_lowercase().as_str() {
        "boursobank" => Ok(boursobank::Importer::import(path)?),
        _ => Err(anyhow::anyhow!("Unknown profile {}", profile.as_ref())),
    }
}

trait Profile {
    fn import<T: AsRef<Path>>(path: T) -> Result<Vec<Data>>;
}

fn parse_date_fmt(date: &str, fmt: &str) -> Result<DateTime<Utc>> {
    crate::cli::naive_date_to_utc(NaiveDate::parse_from_str(date, fmt)?)
}
