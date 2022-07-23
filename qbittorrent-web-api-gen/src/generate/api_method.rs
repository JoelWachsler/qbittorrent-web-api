use case::CaseExt;
use proc_macro2::{Ident, TokenStream};
use quote::quote;

use crate::parser;

use super::util;

impl parser::ApiMethod {
    pub fn structs(&self) -> TokenStream {
        let objects = self.types.objects();
        let structs = objects.iter().map(|obj| obj.generate_struct());

        quote! {
            #(#structs)*
        }
    }

    pub fn enums(&self) -> TokenStream {
        let enums = self.types.enums();
        let generated_enums = enums.iter().map(|e| e.generate());

        quote! {
            #(#generated_enums)*
        }
    }

    pub fn name_snake(&self) -> Ident {
        util::to_ident(&self.name.to_snake())
    }
}
