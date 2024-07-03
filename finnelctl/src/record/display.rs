use std::borrow::Cow;

use finnel::{Entity, record::FullRecord, Record};

use tabled::Tabled;

#[derive(derive_more::From)]
pub struct RecordToDisplay(FullRecord);

impl Tabled for RecordToDisplay {
    const LENGTH: usize = 7;

    fn fields(&self) -> Vec<Cow<'_, str>> {
        vec![
            self.id(),
            self.amount(),
            self.0.record.mode().to_string().into(),
            self.0.record.operation_date().date_naive().to_string().into(),
            self.0.record.details().into(),
            "".into(),
            "".into(),
        ]
    }

    fn headers() -> Vec<Cow<'static, str>> {
        vec![
            "id".into(),
            "amount".into(),
            "mode".into(),
            "operation date".into(),
            "details".into(),
            "category".into(),
            "merchant".into(),
        ]
    }
}

impl RecordToDisplay {
    fn id(&self) -> Cow<'_, str> {
        if let Some(id) = self.0.record.id() {
            id.value().to_string().into()
        } else {
            Default::default()
        }
    }

    fn amount(&self) -> Cow<'_, str> {
        let mut amount = self.0.record.amount();
        amount.0.set_sign_negative(self.0.record.direction().is_debit());

        amount.to_string().into()
    }
}
