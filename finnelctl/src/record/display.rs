use std::borrow::Cow;

use finnel::{Entity, Record};

use tabled::Tabled;

#[derive(derive_more::From)]
pub struct RecordToDisplay(Record);

impl Tabled for RecordToDisplay {
    const LENGTH: usize = 7;

    fn fields(&self) -> Vec<Cow<'_, str>> {
        vec![
            self.id(),
            self.amount(),
            self.0.mode().to_string().into(),
            self.0.operation_date().date_naive().to_string().into(),
            self.0.details().into(),
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
        if let Some(id) = self.0.id() {
            id.value().to_string().into()
        } else {
            Default::default()
        }
    }

    fn amount(&self) -> Cow<'_, str> {
        let mut amount = self.0.amount();
        amount.0.set_sign_negative(self.0.direction().is_debit());

        amount.to_string().into()
    }
}
