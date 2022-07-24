use case::CaseExt;
use proc_macro2::Ident;

use crate::parser;

use super::util;

impl parser::ApiMethod {
    pub fn name_snake(&self) -> Ident {
        util::to_ident(&self.name.to_snake())
    }
}
