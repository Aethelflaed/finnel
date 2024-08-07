use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use super::{parse_date_fmt, parse_decimal, Importer, Options, Profile, RecordToImport};

use finnel::prelude::*;

use anyhow::Result;
use regex::Regex;

pub struct Logseq {
    entries: BTreeSet<PathBuf>,
    regex: Regex,
}

const FORMAT: &str = "%Y_%m_%d.md";

impl Logseq {
    pub fn new(options: &Options) -> Result<Self> {
        let mut entries = BTreeSet::new();
        let from = options.from.map(|d| d.format(FORMAT).to_string());
        let to = options.to.map(|d| d.format(FORMAT).to_string());

        for result in options.file.read_dir()? {
            let entry = result?;
            if entry.file_type()?.is_file() {
                if let Some(ref from) = from {
                    if entry.file_name().into_string().as_ref() < Ok(from) {
                        continue;
                    }
                }
                if let Some(ref to) = to {
                    if entry.file_name().into_string().as_ref() > Ok(to) {
                        continue;
                    }
                }
                entries.insert(entry.path());
            }
        }

        Ok(Logseq {
            entries,
            ..Self::empty()?
        })
    }

    fn empty() -> Result<Self> {
        Ok(Logseq {
            entries: Default::default(),
            regex: Regex::new(
                r#"(?xm)
                ^
                -[[:blank:]]*
                (?:DONE[[:blank:]]*)?
                (?<sign>[+-]?)
                (?<amount>\d+(?:[,.]\d+)?)
                (?<currency>[€])
                [[:blank:]]*
                (?<details>(?:"[^"\r\n]+")|[^\r\n-]*)
                (?:
                    [[:blank:]]*-[[:blank:]]*
                    (?<category>(?:"[^"\r\n]+")|[^\r\n-]*)
                    (?:
                        [[:blank:]]*-[[:blank:]]*
                        (?<merchant>[^\r\n]*)
                    )?
                )?
                [[:blank:]]*
                $"#,
            )?,
        })
    }

    pub fn read(&self, importer: &mut Importer, path: &Path) -> Result<()> {
        let Some(Ok(date)) = path
            .file_name()
            .and_then(|os_str| os_str.to_str())
            .map(|date| parse_date_fmt(date, FORMAT))
        else {
            anyhow::bail!("Unable to parse date from {}", path.display());
        };
        let content = std::fs::read_to_string(path)?;

        for captures in self.regex.captures_iter(&content) {
            match &captures["currency"] {
                "€" => {}
                _ => anyhow::bail!("Unknown currency {}", &captures["currency"]),
            }

            let category = captures.name("category").map(|m| m.as_str()).unwrap_or("");
            let merchant = captures.name("merchant").map(|m| m.as_str()).unwrap_or("");

            let record = RecordToImport {
                operation_date: date,
                value_date: date,
                amount: parse_decimal(&captures["amount"])?,
                direction: match &captures["sign"] {
                    "" | "+" => Direction::Debit,
                    "-" => Direction::Credit,
                    _ => anyhow::bail!("Unknown sign {}", &captures["sign"]),
                },
                details: captures["details"].trim().to_string(),
                category_name: category.trim().to_string(),
                merchant_name: merchant.trim().to_string(),
                ..Default::default()
            };

            importer.add_category(&record.category_name)?;
            importer.add_merchant(&record.merchant_name)?;

            importer.add_record(record)?;
        }

        Ok(())
    }
}

impl Profile for Logseq {
    fn run(&mut self, importer: &mut Importer) -> Result<()> {
        for path in &self.entries {
            self.read(importer, path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::import::tests::with_default_importer;
    use crate::test::prelude::assert_eq;
    use std::{fs::File, io::Write};

    #[test]
    fn test() -> Result<()> {
        with_default_importer(|importer| {
            let conn = &mut importer.options.config.database()?;
            let logseq = Logseq::empty()?;

            let dir = importer.options.config.data_dir.as_path();
            let path = dir.join("2024_07_31.md");
            {
                let mut file = File::create(path.as_path())?;
                writeln!(file, "- 2.5€ cookie - snack - roc en stock")?;
                writeln!(file, "- 2.5 cookie - snack - roc en stock")?;
                writeln!(file, "- 10h50 RDV")?;
                writeln!(file, "blabla")?;
                writeln!(file, "- TODO 1.5€ cookie - snack - roc en stock ")?;
                writeln!(file, "- DONE +3,5€ cookie - snack - roc en stock")?;
                writeln!(file, "- -10€ avance")?;
                writeln!(file, "- 5€ - beer")?;
                writeln!(file, "- 5€ -- mc do")?;
            }

            logseq.read(importer, path.as_path())?;

            let record = Record::find(conn, 1)?;
            assert_eq!(Decimal::new(25, 1), record.amount);
            assert_eq!("cookie", record.details);
            assert_eq!(
                Some("snack"),
                record.fetch_category(conn)?.map(|c| c.name).as_deref()
            );
            assert_eq!(
                Some("roc en stock"),
                record.fetch_merchant(conn)?.map(|m| m.name).as_deref()
            );
            assert_eq!(
                parse_date_fmt("2024-07-31", "%Y-%m-%d")?,
                record.operation_date
            );
            assert_eq!(parse_date_fmt("2024-07-31", "%Y-%m-%d")?, record.value_date);
            assert_eq!(Direction::Debit, record.direction);

            let record = Record::find(conn, 2)?;
            assert_eq!(Decimal::new(35, 1), record.amount);
            assert_eq!(Direction::Debit, record.direction);

            let record = Record::find(conn, 3)?;
            assert_eq!(Decimal::new(10, 0), record.amount);
            assert_eq!("avance", record.details);
            assert_eq!(Direction::Credit, record.direction);
            assert!(record.category_id.is_none());
            assert!(record.merchant_id.is_none());

            let record = Record::find(conn, 4)?;
            assert_eq!(Decimal::new(5, 0), record.amount);
            assert_eq!("", record.details);
            assert_eq!(
                Some("beer"),
                record.fetch_category(conn)?.map(|c| c.name).as_deref()
            );
            assert!(record.merchant_id.is_none());

            let record = Record::find(conn, 5)?;
            assert_eq!(Decimal::new(5, 0), record.amount);
            assert_eq!("", record.details);
            assert!(record.category_id.is_none());
            assert_eq!(
                Some("mc do"),
                record.fetch_merchant(conn)?.map(|c| c.name).as_deref()
            );

            Ok(())
        })
    }
}
