use crate::{
    prelude::*,
    resolved::{mapmapmap, mapmapresolve},
    schema::records,
};

use chrono::{offset::Utc, DateTime};
use diesel::prelude::*;

#[derive(Default, Clone)]
pub struct ChangeRecord<'a> {
    pub value_date: Option<DateTime<Utc>>,
    pub details: Option<&'a str>,
    pub category: Option<Option<&'a Category>>,
    pub merchant: Option<Option<&'a Merchant>>,
}

impl<'a> ChangeRecord<'a> {
    pub fn save(self, conn: &mut Conn, record: &Record) -> Result<()> {
        self.into_violating_change().save(conn, record)
    }

    pub fn apply(self, conn: &mut Conn, record: &mut Record) -> Result<()> {
        self.into_violating_change().save(conn, record)
    }

    fn into_violating_change(self) -> ViolatingChangeRecord<'a> {
        ViolatingChangeRecord {
            value_date: self.value_date,
            details: self.details,
            category: self.category,
            merchant: self.merchant,
            ..Default::default()
        }
    }
}

/// Like ChangeRecord, but allows violating changes, such as updating the record amount
#[derive(Default, Clone)]
pub struct ViolatingChangeRecord<'a> {
    pub amount: Option<Decimal>,
    pub operation_date: Option<DateTime<Utc>>,
    pub value_date: Option<DateTime<Utc>>,
    pub direction: Option<Direction>,
    pub mode: Option<Mode>,
    pub details: Option<&'a str>,
    pub category: Option<Option<&'a Category>>,
    pub merchant: Option<Option<&'a Merchant>>,
}

impl<'a> ViolatingChangeRecord<'a> {
    pub fn save(self, conn: &mut Conn, record: &Record) -> Result<()> {
        self.into_resolved(conn)?.validate(conn, record)?.save(conn)
    }

    pub fn apply(self, conn: &mut Conn, record: &mut Record) -> Result<()> {
        let resolved = self.into_resolved(conn)?;
        let changeset = resolved.as_changeset();
        resolved.validate(conn, record)?.save(conn)?;

        if let Some(value) = changeset.amount {
            record.amount = value;
        }
        if let Some(value) = changeset.operation_date {
            record.operation_date = value;
        }
        if let Some(value) = changeset.value_date {
            record.value_date = value;
        }
        if let Some(value) = changeset.direction {
            record.direction = value;
        }
        if let Some(value) = changeset.mode {
            record.mode = value;
        }
        if let Some(value) = changeset.details {
            record.details = value.to_string();
        }
        if let Some(value) = changeset.category_id {
            record.category_id = value;
        }
        if let Some(value) = changeset.merchant_id {
            record.merchant_id = value;
        }

        Ok(())
    }

    pub fn into_resolved(self, conn: &mut Conn) -> Result<ResolvedChangeRecord<'a>> {
        Ok(ResolvedChangeRecord {
            amount: self.amount,
            operation_date: self.operation_date,
            value_date: self.value_date,
            direction: self.direction,
            mode: self.mode,
            details: self.details,
            category: mapmapresolve(conn, self.category)?,
            merchant: mapmapresolve(conn, self.merchant)?,
        })
    }
}

pub struct ResolvedChangeRecord<'a> {
    pub amount: Option<Decimal>,
    pub operation_date: Option<DateTime<Utc>>,
    pub value_date: Option<DateTime<Utc>>,
    pub direction: Option<Direction>,
    pub mode: Option<Mode>,
    pub details: Option<&'a str>,
    pub category: Option<Option<Resolved<'a, Category>>>,
    pub merchant: Option<Option<Resolved<'a, Merchant>>>,
}

impl<'a> ResolvedChangeRecord<'a> {
    pub fn validate(
        self,
        _conn: &mut Conn,
        record: &'a Record,
    ) -> Result<ValidatedChangeRecord<'a>> {
        // nothing to do?

        Ok(ValidatedChangeRecord(record, self.as_changeset()))
    }

    pub fn as_changeset(&self) -> RecordChangeset<'a> {
        RecordChangeset {
            amount: self.amount,
            operation_date: self.operation_date,
            value_date: self.value_date,
            direction: self.direction,
            mode: self.mode,
            details: self.details,
            category_id: mapmapmap(&self.category, |c| c.id),
            merchant_id: mapmapmap(&self.merchant, |m| m.id),
        }
    }
}

pub struct ValidatedChangeRecord<'a>(&'a Record, RecordChangeset<'a>);

impl<'a> ValidatedChangeRecord<'a> {
    pub fn save(self, conn: &mut Conn) -> Result<()> {
        diesel::update(self.0).set(self.1).execute(conn)?;
        Ok(())
    }
}

#[derive(Default, Clone, AsChangeset)]
#[diesel(table_name = records)]
pub struct RecordChangeset<'a> {
    #[diesel(serialize_as = crate::db::Decimal)]
    pub amount: Option<Decimal>,
    pub operation_date: Option<DateTime<Utc>>,
    pub value_date: Option<DateTime<Utc>>,
    pub direction: Option<Direction>,
    pub mode: Option<Mode>,
    pub details: Option<&'a str>,
    pub category_id: Option<Option<i64>>,
    pub merchant_id: Option<Option<i64>>,
}
