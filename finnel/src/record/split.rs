use crate::{
    prelude::*,
    record::new::InsertableRecord,
    resolved::{mapmap, mapmapresolve},
    schema::records,
};
use diesel::prelude::*;

#[derive(Default)]
pub struct SplitRecord<'a> {
    pub amount: Decimal,
    pub details: Option<&'a str>,
    pub category: Option<Option<&'a Category>>,
}

impl<'a> SplitRecord<'a> {
    pub fn save(self, conn: &mut Conn, record: &Record) -> Result<Record> {
        self.into_resolved(conn)?.validate(conn, record)?.save(conn)
    }

    pub fn apply(self, conn: &mut Conn, record: &mut Record) -> Result<Record> {
        let resolved = self.into_resolved(conn)?;
        let changeset = resolved.as_changeset(record);
        let split = resolved.validate(conn, record)?.save(conn)?;

        record.amount = changeset.amount;

        Ok(split)
    }

    pub fn into_resolved(self, conn: &mut Conn) -> Result<ResolvedSplitRecord<'a>> {
        Ok(ResolvedSplitRecord {
            amount: self.amount,
            details: self.details,
            category: mapmapresolve(conn, self.category)?,
        })
    }
}

pub struct ResolvedSplitRecord<'a> {
    pub amount: Decimal,
    pub details: Option<&'a str>,
    pub category: Option<Option<Resolved<'a, Category>>>,
}

impl<'a> ResolvedSplitRecord<'a> {
    pub fn validate(
        &'a self,
        _conn: &mut Conn,
        record: &'a Record,
    ) -> Result<ValidatedSplitRecord<'a>> {
        if self.amount >= record.amount {
            return Err(Error::Invalid(format!(
                "Unable to split an amount of {} from {}",
                self.amount, record.amount
            )));
        }

        Ok(ValidatedSplitRecord(
            record,
            self.as_changeset(record),
            self.as_insertable(record),
        ))
    }

    pub fn as_changeset(&self, record: &Record) -> SplitRecordChangeset {
        SplitRecordChangeset {
            amount: record.amount - self.amount,
        }
    }

    pub fn as_insertable(&'a self, record: &'a Record) -> InsertableRecord<'a> {
        let category_id = if let Some(wrapped_category) = &self.category {
            mapmap(wrapped_category, |c| c.id)
        } else {
            record.category_id
        };

        InsertableRecord {
            account_id: record.account_id,
            amount: self.amount,
            currency: record.currency,
            operation_date: record.operation_date,
            value_date: record.value_date,
            direction: record.direction,
            mode: record.mode,
            details: self.details.unwrap_or(record.details.as_str()),
            category_id: category_id,
            merchant_id: record.merchant_id,
        }
    }
}

#[derive(Default, Clone, AsChangeset)]
#[diesel(table_name = records)]
pub struct SplitRecordChangeset {
    #[diesel(serialize_as = db::Decimal)]
    pub amount: Decimal,
}

pub struct ValidatedSplitRecord<'a>(&'a Record, SplitRecordChangeset, InsertableRecord<'a>);

impl<'a> ValidatedSplitRecord<'a> {
    pub fn save(self, conn: &mut Conn) -> Result<Record> {
        diesel::update(self.0).set(self.1).execute(conn)?;
        Ok(diesel::insert_into(records::table)
            .values(self.2)
            .returning(Record::as_returning())
            .get_result(conn)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test::prelude::{assert_eq, Result, *};

    #[test]
    fn split() -> Result<()> {
        let conn = &mut test::db()?;

        let account = test::account!(conn, "Cash");
        let category = test::category!(conn, "Category");
        let merchant = test::merchant!(conn, "Merchant");
        let mut record = test::record!(
            conn,
            &account,
            details: "Hello",
            amount: Decimal::new(10, 0),
            category: Some(&category),
            merchant: Some(&merchant)
        );

        let split: Record = SplitRecord {
            amount: Decimal::new(5, 0),
            ..Default::default()
        }
        .save(conn, &record)?;

        record.reload(conn)?;
        assert_eq!(Decimal::new(5, 0), record.amount);
        assert_eq!(Decimal::new(5, 0), split.amount);

        assert_eq!("Hello", split.details.as_str());
        assert_eq!(Some(category.id), split.category_id);
        assert_eq!(Some(merchant.id), split.merchant_id);

        let new_category = test::category!(conn, "New Category");
        let split = SplitRecord {
            amount: Decimal::new(1, 0),
            details: Some("World"),
            category: Some(Some(&new_category)),
        }
        .apply(conn, &mut record)?;

        assert_eq!(Decimal::new(4, 0), record.amount);
        assert_eq!(Decimal::new(1, 0), split.amount);

        assert_eq!("World", split.details.as_str());
        assert_eq!(Some(new_category.id), split.category_id);

        Ok(())
    }

    #[test]
    fn invalid() -> Result<()> {
        let conn = &mut test::db()?;

        let account = test::account!(conn, "Cash");
        let mut record = test::record!(conn, &account, amount: Decimal::new(5, 0));

        assert!(SplitRecord {
            amount: Decimal::new(5, 0),
            ..Default::default()
        }
        .save(conn, &mut record)
        .is_err());

        Ok(())
    }
}
