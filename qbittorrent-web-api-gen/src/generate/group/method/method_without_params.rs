use crate::{generate::util, parser};

use super::{method_builder::MethodBuilder, return_type::create_return_type};

pub fn create_method_without_params(
    group: &parser::ApiGroup,
    method: &parser::ApiMethod,
    method_name: proc_macro2::Ident,
    url: &str,
) -> (proc_macro2::TokenStream, Option<proc_macro2::TokenStream>) {
    let res = match create_return_type(group, method) {
        Some((return_type_name, return_type)) => (
            MethodBuilder::new(&method_name, url)
                .return_type(&return_type_name)
                .build(),
            Some(return_type),
        ),
        None => (
            MethodBuilder::new(&method_name, url).build(),
            // assume that all methods without a return type returns a string
            None,
        ),
    };

    (util::add_docs(&method.description, res.0), res.1)
}
