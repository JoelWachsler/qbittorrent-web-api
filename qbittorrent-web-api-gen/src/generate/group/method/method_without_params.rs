use crate::{generate::util, parser};
use quote::quote;

use super::return_type::create_return_type;

pub fn create_method_without_params(
    group: &parser::ApiGroup,
    method: &parser::ApiMethod,
    method_name: proc_macro2::Ident,
    url: &str,
) -> (proc_macro2::TokenStream, Option<proc_macro2::TokenStream>) {
    let res = match create_return_type(group, method) {
        Some((return_type_name, return_type)) => (
            quote! {
                pub async fn #method_name(&self) -> Result<#return_type_name> {
                    let res = self.auth
                        .authenticated_client(#url)
                        .send()
                        .await?
                        .json::<#return_type_name>()
                        .await?;

                    Ok(res)
                }
            },
            Some(return_type),
        ),
        None => (
            quote! {
                pub async fn #method_name(&self) -> Result<String> {
                    let res = self.auth
                        .authenticated_client(#url)
                        .send()
                        .await?
                        .text()
                        .await?;

                    Ok(res)
                }
            },
            // assume that all methods without a return type returns a string
            None,
        ),
    };

    (util::add_docs(&method.description, res.0), res.1)
}
