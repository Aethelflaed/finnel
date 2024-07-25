use std::path::Path;
use std::str::FromStr;

use super::{parse_date_fmt, Data, Profile, RecordToImport};

use finnel::prelude::*;

use anyhow::Result;
use chrono::{offset::Utc, DateTime};

pub struct Importer;

impl Profile for Importer {
    fn import<T: AsRef<Path>>(path: T) -> Result<Data> {
        let mut reader = csv::ReaderBuilder::new().delimiter(b';').from_path(path)?;

        let mut records = Vec::new();

        for result in reader.records() {
            let row = result?;

            if row.len() != 12 {
                anyhow::bail!("Incorrect number of field in CSV");
            }

            let mut operation_date = Self::parse_date(row.get(0).unwrap())?;
            let value_date = Self::parse_date(row.get(1).unwrap())?;
            let mut details = row.get(2).unwrap();
            let mut mode = Mode::Direct(PaymentMethod::Empty);
            let mut category_name = row.get(3).unwrap();
            let merchant_name = row.get(5).unwrap();
            let amount = Self::parse_decimal(row.get(6).unwrap())?;

            if details.starts_with("CARTE ") || details.starts_with("AVOIR ") {
                let payment_method;
                operation_date = parse_date_fmt(&details[6..14], "%d/%m/%y")?;
                (details, payment_method) = Self::strip_cb_suffix(&details[15..]);
                mode = Mode::Direct(payment_method);
            } else if details.starts_with("VIR ") {
                mode = Mode::Transfer;
                details = &details[4..];
                if let Some(value) = details.strip_prefix("INST ") {
                    details = value;
                }
            } else if details.starts_with("RETRAIT DAB ") {
                let payment_method;
                operation_date = parse_date_fmt(&details[12..20], "%d/%m/%y")?;
                (details, payment_method) = Self::strip_cb_suffix(&details[21..]);
                mode = Mode::Atm(payment_method);
            }

            if category_name == "Non catégorisé" {
                category_name = "";
            }

            let direction = if amount.is_sign_negative() {
                Direction::Debit
            } else {
                Direction::Credit
            };

            records.push(RecordToImport {
                amount: amount.abs(),
                operation_date,
                value_date,
                direction,
                mode,
                details: details.to_string(),
                category_name: category_name.to_string(),
                merchant_name: merchant_name.to_string(),
            });
        }

        Ok(Data::new(records))
    }
}

impl Importer {
    fn parse_date(date: &str) -> Result<DateTime<Utc>> {
        parse_date_fmt(date, "%d/%m/%Y")
    }

    fn parse_decimal(number: &str) -> Result<Decimal> {
        Ok(Decimal::from_str(
            number.replace(",", ".").replace(" ", "").as_str(),
        )?)
    }

    fn strip_cb_suffix(mut details: &str) -> (&str, PaymentMethod) {
        let mut count = 0;
        let mut cb = [' ', ' ', ' ', ' '];

        details = details.trim_end_matches(|c: char| {
            count += 1;
            match count {
                1..=4 => {
                    cb[4 - count] = c;
                    c.is_ascii_digit()
                }
                5 => c == '*',
                6 => c == 'B',
                7 => c == 'C',
                8 => c == ' ',
                _ => false,
            }
        });

        (
            details,
            PaymentMethod::CardLast4Digit(cb[0], cb[1], cb[2], cb[3]),
        )
    }
}
