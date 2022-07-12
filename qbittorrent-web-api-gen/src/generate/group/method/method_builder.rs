use quote::quote;

use crate::generate::util;

pub struct MethodBuilder {
    method_name: syn::Ident,
    url: String,
    auth_module_path: proc_macro2::TokenStream,
    return_type: Option<proc_macro2::TokenStream>,
    description: Option<String>,
}

impl MethodBuilder {
    pub fn new(
        method_name: &syn::Ident,
        url: &str,
        auth_module_path: proc_macro2::TokenStream,
    ) -> Self {
        Self {
            method_name: method_name.clone(),
            url: url.to_string(),
            auth_module_path,
            return_type: None,
            description: None,
        }
    }

    pub fn return_type(mut self, value: &proc_macro2::TokenStream) -> Self {
        self.return_type = Some(value.clone());
        self
    }

    pub fn description(mut self, value: &Option<String>) -> Self {
        self.description = value.clone();
        self
    }

    pub fn build(&self) -> proc_macro2::TokenStream {
        let method_name = &self.method_name;
        let (return_type, parse_type) = match &self.return_type {
            Some(t) => (t.clone(), quote! { .json::<#t>() }),
            None => (quote! { String }, quote! { .text() }),
        };
        let url = &self.url;
        let auth_module_path = &self.auth_module_path;

        util::add_docs(
            &self.description,
            quote! {
                pub async fn #method_name(&self) -> Result<#return_type> {
                    let res = #auth_module_path
                        .authenticated_client(#url)
                        .send()
                        .await?
                        #parse_type
                        .await?;

                    Ok(res)
                }
            },
        )
    }
}
