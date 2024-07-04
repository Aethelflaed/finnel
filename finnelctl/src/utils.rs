use anyhow::Result;
use chrono::{
    offset::{Local, MappedLocalTime, Utc},
    DateTime, NaiveDate, TimeZone,
};

pub fn naive_date_to_utc(date: NaiveDate) -> Result<DateTime<Utc>> {
    match Local.from_local_datetime(&date.and_hms_opt(12, 0, 0).unwrap()) {
        MappedLocalTime::Single(date) => Ok(date.into()),
        MappedLocalTime::Ambiguous(date, _) => Ok(date.into()),
        MappedLocalTime::None => {
            anyhow::bail!("Impossible to map local date to UTC");
        }
    }
}
