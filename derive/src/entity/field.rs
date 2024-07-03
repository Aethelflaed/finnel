use syn::spanned::Spanned;
use syn::{Error, Ident, LitStr, Result};

pub struct Field {
    syn_field: syn::Field,
    ident: Ident,
    default: Option<LitStr>,
}

impl Field {
    pub fn read(input: &syn::Field) -> Result<Field> {
        let mut field = Field {
            syn_field: input.clone(),
            ident: input.ident.clone().unwrap(),
            default: None,
        };

        if let Some(attr) = input
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("field"))
        {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("default") {
                    field.default = Some(meta.value()?.parse()?);
                    return Ok(());
                }

                Err(meta.error("unrecognized query attribute"))
            })?;
        }

        Ok(field)
    }

    pub fn error(&self, message: &str) -> Error {
        Error::new(self.syn_field.span(), message)
    }

    pub fn ident(&self) -> &Ident {
        &self.ident
    }

    pub fn name(&self) -> String {
        self.ident.to_string()
    }
}
