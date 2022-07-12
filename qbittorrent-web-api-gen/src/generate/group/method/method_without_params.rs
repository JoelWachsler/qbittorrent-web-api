use crate::{generate::util, parser};
use quote::quote;

use super::return_type::create_return_type;

struct MethodBuilder {
    method_name: syn::Ident,
    url: String,
    return_type: Option<proc_macro2::TokenStream>,
}

impl MethodBuilder {
    fn new(method_name: &syn::Ident, url: &str) -> Self {
        Self {
            method_name: method_name.clone(),
            url: url.to_string(),
            return_type: None,
        }
    }

    fn return_type(mut self, value: &proc_macro2::TokenStream) -> Self {
        self.return_type = Some(value.clone());
        self
    }

    fn build(&self) -> proc_macro2::TokenStream {
        let method_name = &self.method_name;
        let (return_type, parse_type) = match &self.return_type {
            Some(t) => (t.clone(), quote! { .json::<#t>() }),
            None => (quote! { String }, quote! { .text() }),
        };
        let url = &self.url;

        quote! {
            pub async fn #method_name(&self) -> Result<#return_type> {
                let res = self.auth
                    .authenticated_client(#url)
                    .send()
                    .await?
                    #parse_type
                    .await?;

                Ok(res)
            }
        }
    }
}

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
