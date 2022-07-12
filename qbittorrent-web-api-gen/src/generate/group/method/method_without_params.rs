use quote::quote;

use super::{method_builder::MethodBuilder, return_type::create_return_type};
use crate::parser;

pub fn create_method_without_params(
    group: &parser::ApiGroup,
    method: &parser::ApiMethod,
    method_name: proc_macro2::Ident,
    url: &str,
) -> (proc_macro2::TokenStream, Option<proc_macro2::TokenStream>) {
    let builder = MethodBuilder::new(&method_name, url, quote! { self.auth })
        .description(&method.description);

    match create_return_type(group, method) {
        Some((return_type_name, return_type)) => (
            builder.return_type(&return_type_name).build(),
            Some(return_type),
        ),
        None => (
            builder.build(),
            // assume that all methods without a return type returns a string
            None,
        ),
    }
}
