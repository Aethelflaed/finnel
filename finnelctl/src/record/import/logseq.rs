use std::path::PathBuf;

use super::{parse_date_fmt, Importer, Options, Profile, RecordToImport};

use finnel::prelude::*;

use anyhow::Result;
use chrono::{offset::Utc, DateTime};

pub struct Logseq {
    path: PathBuf,
    from: Option<String>,
    to: Option<String>,
}

impl Logseq {
    pub fn new(options: &Options) -> Result<Self> {
        let format = "%Y_%m_%d.md";

        Ok(Logseq {
            path: options.file.clone(),
            from: options.from.map(|d| d.format(format).to_string()),
            to: options.to.map(|d| d.format(format).to_string()),
        })
    }
}

impl Profile for Logseq {
    fn run(&mut self, importer: &mut Importer) -> Result<()> {
        Ok(())
    }
}
