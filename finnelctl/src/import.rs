use std::collections::HashMap;
use std::str::FromStr;

use crate::cli::import::*;
use crate::config::Config;

use finnel::{category::NewCategory, merchant::NewMerchant, prelude::*, record::NewRecord};

use anyhow::Result;
use chrono::NaiveDate;
use tabled::builder::Builder as TableBuilder;

mod profile;
use profile::{Information, Profile};

mod options;
use options::Options;

mod boursobank;
use boursobank::Boursobank;
mod logseq;
use logseq::Logseq;

type MerchantWithDefaultCategory = (Merchant, Option<Category>);

pub struct Importer<'a> {
    options: Options<'a>,
    pub records: Vec<Record>,
    categories: HashMap<String, Category>,
    merchants: HashMap<String, MerchantWithDefaultCategory>,
    conn: &'a mut Conn,
    account: Account,
}

#[derive(Default, Clone)]
pub struct RecordToImport {
    pub operation_date: NaiveDate,
    pub value_date: NaiveDate,
    pub amount: Decimal,
    pub direction: Direction,
    pub mode: Mode,
    pub details: String,
    pub category_name: String,
    pub merchant_name: String,
}

fn parse_date_fmt(date: &str, fmt: &str) -> Result<NaiveDate> {
    Ok(NaiveDate::parse_from_str(date, fmt)?)
}

fn parse_decimal(number: &str) -> Result<Decimal> {
    Ok(Decimal::from_str(
        number.replace(",", ".").replace(" ", "").as_str(),
    )?)
}

pub fn run(config: &Config, command: &Command) -> Result<()> {
    let conn = &mut config.database()?;

    let options = Options::try_from(command, config)?;

    if options.has_configuration_action() {
        options.configure(conn)?;
        return Ok(());
    }

    conn.transaction(|conn| {
        let Importer {
            records,
            options,
            categories,
            merchants,
            ..
        } = {
            let mut importer = Importer::new(conn, options)?;
            importer.run().map(|_| importer)
        }?;

        let categories_by_id = categories
            .values()
            .map(|category| (category.id, category))
            .collect::<HashMap<i64, &Category>>();

        let merchants_by_id = merchants
            .values()
            .map(|(merchant, _)| (merchant.id, merchant))
            .collect::<HashMap<i64, &Merchant>>();

        if options.print {
            let mut builder = TableBuilder::new();
            table_push_row!(
                builder,
                std::marker::PhantomData::<(Record, Option<Category>, Option<Merchant>)>
            );

            for record in records {
                let category = record.category_id.as_ref().map(|id| categories_by_id[id]);
                let merchant = record.merchant_id.as_ref().map(|id| merchants_by_id[id]);

                table_push_row!(builder, (record, category, merchant));
            }
            println!("{}", builder.build());
        }

        if options.pretend {
            anyhow::bail!("No records were saved as we are pretending");
        }

        Ok(())
    })
}

impl<'a> Importer<'a> {
    fn new(conn: &'a mut Conn, options: Options<'a>) -> Result<Self> {
        Ok(Importer {
            account: options.account(conn)?,
            options,
            records: Default::default(),
            categories: Default::default(),
            merchants: Default::default(),
            conn,
        })
    }

    fn run(&mut self) -> Result<()> {
        self.options.new_profile()?.run(self)
    }

    fn add_record(&mut self, import: RecordToImport) -> Result<Option<&Record>> {
        if let Some(date) = self.options.from {
            if import.operation_date < date {
                return Ok(None);
            }
        }
        if let Some(date) = self.options.to {
            if import.operation_date > date {
                return Ok(None);
            }
        }

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
                ..NewRecord::new(&self.account)
            }
            .save(self.conn)?,
        );

        let record = self
            .records
            .last()
            .ok_or(anyhow::anyhow!("No last record?"))?;

        self.options
            .set_last_imported(Some(record.operation_date))?;

        Ok(Some(record))
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
                Err(e) if e.is_not_found() => NewCategory::new(name).save(self.conn)?,
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
                Err(e) if e.is_not_found() => NewMerchant::new(name).save(self.conn)?,
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

    pub fn with_default_importer<F, R>(function: F) -> Result<R>
    where
        F: FnOnce(&mut Importer) -> Result<R>,
    {
        with_config(|config| {
            let options = Options::new(config);
            with_importer(options, function)
        })
    }

    pub fn with_importer<F, R>(options: Options, function: F) -> Result<R>
    where
        F: FnOnce(&mut Importer) -> Result<R>,
    {
        let conn = &mut options.config.database()?;
        let _account = test::account(conn, "Importer")?;

        options.profile_info.set_configuration(
            options.config,
            ConfigurationKey::DefaultAccount,
            Some("Importer"),
        )?;

        function(&mut Importer::new(conn, options)?)
    }

    #[test]
    fn add_record() -> Result<()> {
        with_default_importer(|importer| {
            let conn = &mut importer.options.config.database()?;
            let account_id = importer.options.account(conn)?.id;

            let date = chrono::Utc::now().date_naive();
            let mut record_to_import = RecordToImport {
                amount: Decimal::new(314, 2),
                operation_date: date,
                value_date: date,
                details: "Hello World".to_string(),
                category_name: "restaurant".to_string(),
                merchant_name: "chariot".to_string(),
                ..Default::default()
            };

            let record = importer.add_record(record_to_import.clone())?.unwrap();

            // the category and merchant were not added to the importer first so it doesn't work
            assert!(record.category_id.is_none());
            assert!(record.merchant_id.is_none());
            assert_eq!(account_id, record.account_id);

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

            let record = importer.add_record(record_to_import.clone())?.unwrap();
            assert_eq!(Some(restaurant.id), record.category_id);
            assert_eq!(Some(chariot.id), record.merchant_id);

            record_to_import.category_name = String::new();

            // Use merchant's default_category by default
            let record = importer.add_record(record_to_import)?.unwrap();
            assert_eq!(Some(bar.id), record.category_id);

            Ok(())
        })
    }

    #[test]
    fn add_record_from_to() -> Result<()> {
        with_config(|config| {
            let options = Options {
                from: Some(parse_date_fmt("2024-07-01", "%Y-%m-%d")?),
                to: Some(parse_date_fmt("2024-07-31", "%Y-%m-%d")?),
                profile_info: Information::Boursobank,
                ..Options::new(config)
            };

            with_importer(options, |importer| {
                let date = parse_date_fmt("2024-06-30", "%Y-%m-%d")?;

                let mut record_to_import = RecordToImport {
                    amount: Decimal::new(314, 2),
                    operation_date: date,
                    value_date: chrono::Utc::now().date_naive(),
                    details: "Hello World".to_string(),
                    category_name: "restaurant".to_string(),
                    merchant_name: "chariot".to_string(),
                    ..Default::default()
                };

                assert!(importer.add_record(record_to_import.clone())?.is_none());

                record_to_import.operation_date = parse_date_fmt("2024-08-01", "%Y-%m-%d")?;
                assert!(importer.add_record(record_to_import.clone())?.is_none());

                record_to_import.operation_date = parse_date_fmt("2024-07-01", "%Y-%m-%d")?;
                assert!(importer.add_record(record_to_import)?.is_some());

                assert!(importer.options.last_imported()?.is_some());

                Ok(())
            })
        })
    }

    #[test]
    fn add_get_category() -> Result<()> {
        with_default_importer(|importer| {
            let conn = &mut importer.options.config.database()?;

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
        with_default_importer(|importer| {
            let conn = &mut importer.options.config.database()?;

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

    #[test]
    fn parse_decimal() -> Result<()> {
        assert!(super::parse_decimal("hello").is_err());

        assert_eq!(Decimal::new(314, 2), super::parse_decimal("3,14")?);
        assert_eq!(Decimal::new(314, 2), super::parse_decimal("3.14")?);

        assert_eq!(Decimal::new(65536, 0), super::parse_decimal("65536")?);
        assert_eq!(Decimal::new(65536, 0), super::parse_decimal("65 536")?);
        Ok(())
    }
}
