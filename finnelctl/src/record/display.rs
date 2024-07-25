use std::borrow::Cow;

use finnel::prelude::*;

use tabled::Tabled;

#[derive(derive_more::From)]
pub struct RecordToDisplay(Record, Option<Category>, Option<Merchant>);

impl Tabled for RecordToDisplay {
    const LENGTH: usize = 8;

    fn fields(&self) -> Vec<Cow<'_, str>> {
        vec![
            self.0.id.to_string().into(),
            self.amount(),
            self.0.mode.to_string().into(),
            self.0.operation_date.date_naive().to_string().into(),
            self.0.value_date.date_naive().to_string().into(),
            self.0.details.clone().into(),
            self.category(),
            self.merchant(),
        ]
    }

    fn headers() -> Vec<Cow<'static, str>> {
        vec![
            "id".into(),
            "amount".into(),
            "mode".into(),
            "operation date".into(),
            "value date".into(),
            "details".into(),
            "category".into(),
            "merchant".into(),
        ]
    }
}

impl RecordToDisplay {
    fn amount(&self) -> Cow<'_, str> {
        let mut amount = self.0.amount();
        amount.0.set_sign_negative(self.0.direction.is_debit());

        amount.to_string().into()
    }

    fn category(&self) -> Cow<'_, str> {
        if let Some(category) = &self.1 {
            category.name.clone().into()
        } else {
            Default::default()
        }
    }

    fn merchant(&self) -> Cow<'_, str> {
        if let Some(merchant) = &self.2 {
            merchant.name.clone().into()
        } else {
            Default::default()
        }
    }
}
