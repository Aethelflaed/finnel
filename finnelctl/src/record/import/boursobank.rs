use std::path::Path;
use std::str::FromStr;

use super::{parse_date_fmt, Profile, RecordToImport};
use crate::cli::record::Import as ImportOptions;

use finnel::prelude::*;

use anyhow::Result;
use chrono::{offset::Utc, DateTime};

pub struct Importer<'a> {
    reader: csv::Reader<std::fs::File>,
    _options: &'a ImportOptions,
}

impl<'a> Importer<'a> {
    pub fn new(path: &Path, options: &'a ImportOptions) -> Result<Self> {
        let reader = csv::ReaderBuilder::new().delimiter(b';').from_path(path)?;
        Ok(Importer {
            reader,
            _options: options,
        })
    }
}

impl Profile for Importer<'_> {
    fn run(&mut self, importer: &mut super::Importer) -> Result<()> {
        for result in self.reader.records() {
            let row = result?;

            if row.len() != 12 {
                anyhow::bail!("Incorrect number of field in CSV");
            }

            let mut operation_date = parse_date(row.get(0).unwrap())?;
            let value_date = parse_date(row.get(1).unwrap())?;
            let mut details = row.get(2).unwrap();
            let mut mode = Mode::Direct(PaymentMethod::Empty);
            let mut category_name = row.get(3).unwrap();
            let merchant_name = row.get(5).unwrap();
            let amount = parse_decimal(row.get(6).unwrap())?;

            if details.starts_with("CARTE ") || details.starts_with("AVOIR ") {
                let payment_method;
                operation_date = parse_date_fmt(&details[6..14], "%d/%m/%y")?;
                (details, payment_method) = strip_cb_suffix(&details[15..]);
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
                (details, payment_method) = strip_cb_suffix(&details[21..]);
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

            importer.add_merchant(merchant_name)?;

            let detected_category_name = category_name;
            let category_name = importer
                .get_merchant(merchant_name)
                .map(|(_, category)| category.as_ref().map(|c| c.name.clone()))
                .flatten()
                .unwrap_or_else(|| category_name.to_string());

            if category_name == detected_category_name {
                importer.add_category(&category_name)?;
            }

            importer.add_record(RecordToImport {
                amount: amount.abs(),
                operation_date,
                value_date,
                direction,
                mode,
                details: details.to_string(),
                category_name,
                merchant_name: merchant_name.to_string(),
            })?;
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
