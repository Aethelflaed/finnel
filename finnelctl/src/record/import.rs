use std::collections::HashMap;

use crate::cli::record::Import as ImportOptions;

use finnel::{category::NewCategory, merchant::NewMerchant, prelude::*, record::NewRecord};

use anyhow::Result;
use chrono::{offset::Utc, DateTime, NaiveDate};

mod boursobank;
use boursobank::Boursobank;

type MerchantWithDefaultCategory = (Merchant, Option<Category>);

pub struct Importer<'a> {
    _options: &'a ImportOptions,
    pub records: Vec<Record>,
    categories: HashMap<String, Category>,
    merchants: HashMap<String, MerchantWithDefaultCategory>,
    conn: &'a mut Conn,
    account: &'a Account,
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

trait Profile {
    fn run(&mut self, importer: &mut Importer) -> Result<()>;
}

fn parse_date_fmt(date: &str, fmt: &str) -> Result<DateTime<Utc>> {
    crate::utils::naive_date_to_utc(NaiveDate::parse_from_str(date, fmt)?)
}

pub fn run<'a>(conn: &mut Conn, account: &Account, options: &'a ImportOptions) -> Result<()> {
    let mut profile = match options.profile.to_lowercase().as_str() {
        "boursobank" => Box::new(Boursobank::new(options)?),
        _ => anyhow::bail!("Unknown profile '{}'", options.profile),
    };

    conn.transaction(|conn| {
        let mut importer = Importer {
            _options: options,
            records: Default::default(),
            categories: Default::default(),
            merchants: Default::default(),
            conn: conn,
            account: account,
        };

        profile.run(&mut importer)
    })?;

    Ok(())
}

impl<'a> Importer<'a> {
    fn add_record(&mut self, import: RecordToImport) -> Result<&Record> {
        let conn = &mut self.conn;

        // rust doesn't look into the functions to ascertain we can do something or not, so
        // calling get_category/get_merchant here instead makes the borrow checker unhappy
        // error[E0502]: cannot borrow `*self` as immutable because it is also borrowed as mutable
        let (merchant, category) = if import.merchant_name.is_empty() {
            (None, None)
        } else {
            self.merchants
                .get(&import.merchant_name)
                .map(|(merchant, category)| (Some(merchant), category.as_ref()))
                .unwrap_or((None, None))
        };

        let category = if import.category_name.is_empty() {
            None
        } else {
            self.categories.get(&import.category_name)
        }
        .or(category);

        self.records.push(
            NewRecord {
                amount: import.amount,
                operation_date: import.operation_date,
                value_date: import.value_date,
                direction: import.direction,
                mode: import.mode,
                details: import.details.as_str(),
                category,
                merchant,
                ..NewRecord::new(self.account)
            }
            .save(conn)?,
        );

        Ok(self
            .records
            .last()
            .ok_or(anyhow::anyhow!("No last record?"))?)
    }

    #[allow(dead_code)]
    fn get_category(&self, name: &str) -> Option<&Category> {
        if name.is_empty() {
            None
        } else {
            self.categories.get(name)
        }
    }

    fn add_category(&mut self, name: &str) -> Result<()> {
        if !name.is_empty() && !self.categories.contains_key(name) {
            let category = match Category::find_by_name(self.conn, name) {
                Ok(category) => category,
                Err(Error::NotFound) => NewCategory::new(name).save(self.conn)?,
                Err(e) => return Err(e.into()),
            };

            let category = category.resolve(self.conn)?;

            self.categories.insert(name.to_string(), category);
        }

        Ok(())
    }

    fn get_merchant(&self, name: &str) -> Option<&MerchantWithDefaultCategory> {
        if name.is_empty() {
            None
        } else {
            self.merchants.get(name)
        }
    }

    fn add_merchant(&mut self, name: &str) -> Result<()> {
        if !name.is_empty() && !self.merchants.contains_key(name) {
            let merchant = match Merchant::find_by_name(self.conn, name) {
                Ok(merchant) => merchant,
                Err(Error::NotFound) => NewMerchant::new(name).save(self.conn)?,
                Err(e) => return Err(e.into()),
            };

            let merchant = merchant.resolve(self.conn)?;
            let default_category = merchant.fetch_default_category(self.conn)?;

            self.merchants
                .insert(name.to_string(), (merchant, default_category));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::prelude::{assert_eq, *};

    fn importer<'a>(
        conn: &'a mut Conn,
        account: &'a Account,
        options: &'a ImportOptions,
    ) -> Importer<'a> {
        Importer {
            _options: options,
            records: Default::default(),
            categories: Default::default(),
            merchants: Default::default(),
            conn,
            account,
        }
    }

    #[test]
    fn add_record() -> Result<()> {
        with_temp_dir(|dir| {
            // need two connections, because one is exclusively shared to the importer
            let importer_conn = &mut test::conn_file(dir.child("record_import").path())?;
            let conn = &mut test::conn_file(dir.child("record_import").path())?;

            let account = &test::account(conn, "Cash")?;
            let options = &ImportOptions::default();

            let mut importer = importer(importer_conn, account, options);

            let date = Utc::now();
            let mut record_to_import = RecordToImport {
                amount: Decimal::new(314, 2),
                operation_date: date,
                value_date: date,
                direction: Direction::Debit,
                mode: Mode::default(),
                details: "Hello World".to_string(),
                category_name: "restaurant".to_string(),
                merchant_name: "chariot".to_string(),
            };

            let record = importer.add_record(record_to_import.clone())?;

            // the category and merchant were not added to the importer first so it doesn't work
            assert!(record.category_id.is_none());
            assert!(record.merchant_id.is_none());
            assert_eq!(account.id, record.account_id);

            let restaurant = test::category(conn, "restaurant")?;
            let bar = test::category(conn, "bar")?;
            let mut chariot = test::merchant(conn, "chariot")?;
            finnel::merchant::ChangeMerchant {
                default_category: Some(Some(&bar)),
                ..Default::default()
            }
            .apply(conn, &mut chariot)?;

            importer.add_merchant("chariot")?;
            importer.add_category("restaurant")?;

            let record = importer.add_record(record_to_import.clone())?;
            assert_eq!(Some(restaurant.id), record.category_id);
            assert_eq!(Some(chariot.id), record.merchant_id);

            record_to_import.category_name = String::new();

            // Use merchant's default_category by default
            let record = importer.add_record(record_to_import)?;
            assert_eq!(Some(bar.id), record.category_id);

            Ok(())
        })
    }

    #[test]
    fn add_get_category() -> Result<()> {
        with_temp_dir(|dir| {
            // need two connections, because one is exclusively shared to the importer
            let importer_conn = &mut test::conn_file(dir.child("record_import").path())?;
            let conn = &mut test::conn_file(dir.child("record_import").path())?;

            let account = &test::account(conn, "Cash")?;
            let options = &ImportOptions::default();

            let mut importer = importer(importer_conn, account, options);

            assert!(importer.add_category("").is_ok());
            assert!(importer.add_category("").is_ok());
            assert!(importer.get_category("").is_none());

            assert!(importer.add_category("hotel").is_ok());
            assert!(importer.add_category("hotel").is_ok());
            assert!(importer.get_category("hotel").is_some());

            let mut bars = test::category(conn, "bars")?;
            let bar = test::category(conn, "bar")?;
            finnel::category::ChangeCategory {
                replaced_by: Some(Some(&bar)),
                ..Default::default()
            }
            .apply(conn, &mut bars)?;

            assert!(importer.add_category("bars").is_ok());
            assert!(importer.add_category("bars").is_ok());
            assert_eq!(bar.id, importer.get_category("bars").unwrap().id);

            assert!(importer.get_category("bar").is_none());
            assert!(importer.add_category("bar").is_ok());
            assert_eq!(bar.id, importer.get_category("bar").unwrap().id);

            Ok(())
        })
    }

    #[test]
    fn add_get_merchant() -> Result<()> {
        with_temp_dir(|dir| {
            // need two connections, because one is exclusively shared to the importer
            let importer_conn = &mut test::conn_file(dir.child("record_import").path())?;
            let conn = &mut test::conn_file(dir.child("record_import").path())?;

            let account = &test::account(conn, "Cash")?;
            let options = &ImportOptions::default();

            let mut importer = importer(importer_conn, account, options);

            importer.add_merchant("")?;
            importer.add_merchant("")?;
            assert!(importer.get_merchant("").is_none());

            importer.add_merchant("mc")?;
            importer.add_merchant("mc")?;
            assert!(importer.get_merchant("mc").is_some());
            assert!(importer.get_merchant("mc").unwrap().1.is_none());

            let bar = test::category(conn, "bar")?;
            let mut le_chariot = test::merchant(conn, "le chariot")?;
            let mut chariot = test::merchant(conn, "chariot")?;
            finnel::merchant::ChangeMerchant {
                replaced_by: Some(Some(&chariot)),
                default_category: Some(Some(&bar)),
                ..Default::default()
            }
            .apply(conn, &mut le_chariot)?;

            importer.add_merchant("le chariot")?;
            importer.add_merchant("le chariot")?;
            assert_eq!(
                chariot.id,
                importer.get_merchant("le chariot").unwrap().0.id
            );
            assert!(importer.get_merchant("le chariot").unwrap().1.is_none());

            // Apply the default category on the replacer now
            finnel::merchant::ChangeMerchant {
                default_category: Some(Some(&bar)),
                ..Default::default()
            }
            .apply(conn, &mut chariot)?;

            assert!(importer.get_merchant("chariot").is_none());
            importer.add_merchant("chariot")?;
            assert_eq!(
                bar.id,
                importer
                    .get_merchant("chariot")
                    .unwrap()
                    .1
                    .as_ref()
                    .unwrap()
                    .id
            );

            // We remove the entry to reload it in the cache
            importer.merchants.remove("le chariot");
            importer.add_merchant("le chariot")?;
            // And the default category should be present now
            assert_eq!(
                bar.id,
                importer
                    .get_merchant("le chariot")
                    .unwrap()
                    .1
                    .as_ref()
                    .unwrap()
                    .id
            );

            Ok(())
        })
    }
}
