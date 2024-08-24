use crate::essentials::*;

use std::ops::Range;

use chrono::{Days, Months, NaiveDate};

pub enum Month {
    Calendar { year: i32, month: i32 },
    Until(NaiveDate),
}

impl Month {
    pub fn calendar(year: i32, month: i32) -> Self {
        Self::Calendar { year, month }
    }

    pub fn until(date: NaiveDate) -> Self {
        Self::Until(date)
    }

    pub fn as_date_range(&self) -> Result<Range<NaiveDate>> {
        Ok(match *self {
            Self::Calendar { year, month } => {
                let from = NaiveDate::from_ymd_opt(year, month as u32, 1)
                    .ok_or(Error::InvalidMonth(year, month))?;
                let to = from + Months::new(1);

                from..to
            }
            Self::Until(date) => {
                let to = date + Days::new(1);
                let from = to - Months::new(1);
                from..to
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::prelude::{assert_eq, Result, *};

    #[test]
    fn month_calendar() -> Result<()> {
        let month = Month::calendar(12, -6);
        let result = month.as_date_range();

        assert!(matches!(result, Err(Error::InvalidMonth(12, -6))));

        let month = Month::calendar(2024, 2);
        let range = month.as_date_range()?;

        assert_eq!(NaiveDate::from_ymd_opt(2024, 2, 1), Some(range.start));
        assert_eq!(NaiveDate::from_ymd_opt(2024, 3, 1), Some(range.end));

        Ok(())
    }

    #[test]
    fn month_until() -> Result<()> {
        let date = NaiveDate::from_ymd_opt(2024, 2, 29).unwrap();
        let month = Month::until(date);
        let range = month.as_date_range()?;

        assert_eq!(NaiveDate::from_ymd_opt(2024, 2, 1), Some(range.start));
        assert_eq!(NaiveDate::from_ymd_opt(2024, 3, 1), Some(range.end));

        Ok(())
    }
}
