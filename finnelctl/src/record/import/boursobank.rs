use std::str::FromStr;

use super::{parse_date_fmt, Importer, Profile, RecordToImport};
use crate::cli::record::Import as ImportOptions;

use finnel::prelude::*;

use anyhow::Result;
use chrono::{offset::Utc, DateTime};

pub struct Boursobank<'a> {
    reader: csv::Reader<std::fs::File>,
    _options: &'a ImportOptions,
}

impl<'a> Boursobank<'a> {
    pub fn new(options: &'a ImportOptions) -> Result<Self> {
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(b';')
            .from_path(&options.file)?;

        {
            let headers = reader.headers()?;
            let expected_headers = vec![
                "dateOp",
                "dateVal",
                "label",
                "category",
                "categoryParent",
                "supplierFound",
                "amount",
                "accountNum",
                "accountLabel",
                "accountBalance",
                "comment",
                "pointer",
            ];

            if headers != expected_headers {
                anyhow::bail!("Invalid CSV header, expecting {:?}", expected_headers);
            }
        }

        Ok(Boursobank {
            reader,
            _options: options,
        })
    }
}

impl Profile for Boursobank<'_> {
    fn run(&mut self, importer: &mut Importer) -> Result<()> {
        for result in self.reader.records() {
            let row = result?;

            let mut record = RecordToImport {
                operation_date: parse_date(row.get(0).unwrap())?,
                value_date: parse_date(row.get(1).unwrap())?,
                amount: parse_decimal(row.get(6).unwrap())?,
                mode: Mode::Direct(PaymentMethod::Empty),
                details: row.get(2).unwrap().to_string(),
                category_name: row.get(3).unwrap().to_string(),
                merchant_name: row.get(5).unwrap().to_string(),
                ..Default::default()
            };

            if record.details.starts_with("CARTE ") || record.details.starts_with("AVOIR ") {
                // CARTE DD/MM/YYYY ... CB*WXYZ
                // AVOIR DD/MM/YYYY ... CB*WXYZ
                record.operation_date = parse_date_fmt(&record.details[6..14], "%d/%m/%y")?;
                let payment_method =
                    PaymentMethod::read(&record.details[record.details.len() - 8..], " CB")?;
                record.details = record.details[15..record.details.len() - 8].to_string();
                record.mode = Mode::Direct(payment_method);
            } else if record.details.starts_with("RETRAIT DAB ") {
                // RETRAIT DAB DD/MM/YYYY ... CB*WXYZ
                record.operation_date = parse_date_fmt(&record.details[12..20], "%d/%m/%y")?;
                let payment_method =
                    PaymentMethod::read(&record.details[record.details.len() - 8..], " CB")?;
                record.details = record.details[21..record.details.len() - 8].to_string();
                record.mode = Mode::Atm(payment_method);

                // We don't need either the category or the merchant from Boursobank
                record.category_name = String::new();
                record.merchant_name = String::new();
            } else if record.details.starts_with("VIR ") | record.details.starts_with("PRLV ") {
                // VIR|PRLV INST ...
                // VIR|PRLV SEPA ...
                // VIR|PRLV ...
                record.mode = Mode::Transfer;
                match &record.details[0..4] {
                    "VIR " => record.details = record.details[4..].to_string(),
                    "PRLV" => record.details = record.details[5..].to_string(),
                    _ => {}
                }
                match &record.details[0..5] {
                    "INST " | "SEPA " => record.details = record.details[5..].to_string(),
                    _ => {}
                }

                // We don't need the category from Boursobank
                record.category_name = String::new();
                // If the merchant is empty, use the details
                if record.merchant_name.is_empty() {
                    record.merchant_name = record.details.clone();
                }

                if record.merchant_name.starts_with("virement ") {
                    record.merchant_name = record.merchant_name[9..].to_string();
                    if record.merchant_name.starts_with("interne depuis ") {
                        record.merchant_name = record.merchant_name[15..].to_string();
                    }
                }
            }

            if record.category_name == "Non catégorisé" {
                record.category_name = String::new();
            }

            record.direction = if record.amount.is_sign_negative() {
                Direction::Debit
            } else {
                Direction::Credit
            };
            record.amount = record.amount.abs();

            importer.add_merchant(&record.merchant_name)?;

            let detected_category_name = record.category_name;

            // merchant's default_category takes precedence over the category_name because
            // boursobank's categories are not what we want
            record.category_name = importer
                .get_merchant(&record.merchant_name)
                .map(|(_, category)| category.as_ref().map(|c| c.name.clone()))
                .flatten()
                .unwrap_or_else(|| detected_category_name.clone());

            // If we still end up with the initial category_name, only then do we add it to the
            // importer
            if record.category_name == detected_category_name {
                importer.add_category(&detected_category_name)?;
            }

            importer.add_record(record)?;
        }

        Ok(())
    }
}

fn parse_date(date: &str) -> Result<DateTime<Utc>> {
    parse_date_fmt(date, "%d/%m/%Y")
}

fn parse_decimal(number: &str) -> Result<Decimal> {
    Ok(Decimal::from_str(
        number.replace(",", ".").replace(" ", "").as_str(),
    )?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::prelude::{assert_eq, Result, *};
    use finnel::{category::NewCategory, merchant::NewMerchant};

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
    fn invalid_header() -> Result<()> {
        let csv = "boursobank/invalid_header.csv";

        with_fixtures(&[csv], |dir| {
            let options = ImportOptions {
                file: dir.child(csv).path().to_path_buf(),
                ..ImportOptions::default()
            };
            let result = Boursobank::new(&options);
            assert!(result.is_err());

            Ok(())
        })
    }

    #[test]
    fn import() -> Result<()> {
        let csv = "boursobank/curated.csv";
        with_fixtures(&[csv], |dir| {
            // need two connections, because one is exclusively shared to the importer
            let importer_conn = &mut test::conn_file(dir.child("boursobank.db").path())?;
            let conn = &mut test::conn_file(dir.child("boursobank.db").path())?;

            let account = &test::account(conn, "Cash")?;

            let bar = test::category(conn, "Bar")?;
            let chariot = NewMerchant {
                name: "chariot",
                default_category: Some(&bar),
                ..Default::default()
            }
            .save(conn)?;
            let _le_chariot = NewMerchant {
                name: "le chariot",
                replaced_by: Some(&chariot),
                ..Default::default()
            }
            .save(conn)?;

            let insurances = test::category(conn, "Insurances")?;
            let _assus = NewCategory {
                name: "Assurance habitation et RC",
                replaced_by: Some(&insurances),
                ..Default::default()
            }
            .save(conn)?;

            let music = test::category(conn, "Music")?;
            let spotify = NewMerchant {
                name: "Spotify",
                default_category: Some(&music),
                ..Default::default()
            }
            .save(conn)?;

            let options = ImportOptions {
                file: dir.child(csv).path().to_path_buf(),
                ..ImportOptions::default()
            };
            let mut importer = importer(importer_conn, account, &options);
            let mut profile = Boursobank::new(&options)?;
            profile.run(&mut importer)?;

            assert_eq!(9, importer.records.len());

            let record = &importer.records[0];
            assert_eq!(Some(chariot.id), record.merchant_id);
            assert_eq!(Some(bar.id), record.category_id);
            assert_eq!(
                Mode::Direct(PaymentMethod::CardLast4Digit('1', '2', '3', '4')),
                record.mode
            );
            assert_eq!(Direction::Debit, record.direction);
            assert_eq!("LE CHARIOT", record.details);
            assert_eq!(Decimal::new(55, 1), record.amount);
            assert_eq!(parse_date("27/06/2024")?, record.value_date);
            assert_eq!(parse_date("25/06/2024")?, record.operation_date);

            let record = &importer.records[1];
            assert_eq!(Some(insurances.id), record.category_id);
            assert_eq!(
                "rac insurance qb",
                Merchant::find(conn, record.merchant_id.unwrap())?.name
            );
            assert_eq!(
                Mode::Direct(PaymentMethod::CardLast4Digit('4', '1', '3', '2')),
                record.mode
            );
            assert_eq!(Direction::Credit, record.direction);
            assert_eq!("RAC INSURANCE QB", record.details);
            assert_eq!(Decimal::new(1079, 2), record.amount);
            assert_eq!(parse_date("22/06/2024")?, record.value_date);
            assert_eq!(parse_date("20/06/2024")?, record.operation_date);

            let record = &importer.records[2];
            assert_eq!(None, record.category_id);
            assert!(record.merchant_id.is_some());
            assert_eq!("TRANSFERWISE", record.details);
            assert_eq!(Mode::Transfer, record.mode);
            assert_eq!(Direction::Credit, record.direction);
            assert_eq!(Decimal::new(123456, 2), record.amount);

            let record = &importer.records[3];
            assert_eq!(None, record.category_id);
            assert_eq!(
                "cpam moselle",
                Merchant::find(conn, record.merchant_id.unwrap())?.name
            );
            assert_eq!("CPAM MOSELLE", record.details);
            assert_eq!(Mode::Transfer, record.mode);
            assert_eq!(Decimal::new(5454, 2), record.amount);

            let record = &importer.records[4];
            assert_eq!(None, record.category_id);
            assert!(record.merchant_id.is_some());
            assert_eq!(
                "livret a",
                Merchant::find(conn, record.merchant_id.unwrap())?.name
            );
            assert_eq!("Virement interne depuis LIVRET A", record.details);
            assert_eq!(Mode::Transfer, record.mode);
            assert_eq!(parse_date("29/06/2024")?, record.value_date);
            assert_eq!(parse_date("28/06/2024")?, record.operation_date);

            let record = &importer.records[5];
            assert_eq!(None, record.category_id);
            assert_eq!(None, record.merchant_id);
            assert_eq!("STRASBOURG", record.details);
            assert_eq!(
                Mode::Atm(PaymentMethod::CardLast4Digit('1', '2', '3', '4')),
                record.mode
            );

            let record = &importer.records[6];
            assert_eq!(None, record.category_id);
            assert_eq!(None, record.merchant_id);
            assert_eq!("Spotify", record.details);

            let record = &importer.records[7];
            assert_eq!(Some(music.id), record.category_id);
            assert_eq!(Some(spotify.id), record.merchant_id);
            assert_eq!("Spotify", record.details);

            let record = &importer.records[8];
            assert_eq!(
                "BLOC EN STOCK",
                Merchant::find(conn, record.merchant_id.unwrap())?.name
            );
            assert_eq!("BLOC EN STOCK", record.details);
            assert_eq!(Mode::Transfer, record.mode);
            assert_eq!(Direction::Debit, record.direction);

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
