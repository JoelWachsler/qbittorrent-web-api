use crate::parser;
use proc_macro2::{Ident, TokenStream};
use quote::quote;

use super::util;

#[derive(Debug)]
pub struct GroupMethod<'a> {
    group: &'a parser::ApiGroup,
    method: &'a parser::ApiMethod,
}

impl<'a> GroupMethod<'a> {
    pub fn new(group: &'a parser::ApiGroup, method: &'a parser::ApiMethod) -> Self {
        Self { group, method }
    }

    pub fn generate_method(&self) -> TokenStream {
        let method_name = self.method.name_snake();
        let structs = self.method.structs();
        let enums = self.method.enums();
        let builder = self.generate_request_builder();
        let response_struct = self.generate_response_struct();
        let request_method = self.generate_request_method();

        quote! {
            pub mod #method_name {
                #structs
                #enums
                #builder
                #response_struct
                #request_method
            }
        }
    }

    fn generate_request_method(&self) -> TokenStream {
        let method_name = self.method.name_snake();

        let parameters = self
            .method
            .types
            .mandatory_params()
            .iter()
            .map(|param| param.to_parameter())
            .collect();

        let form_builder = self.mandatory_parameters_as_form_builder();

        let method_impl = if self.method.types.optional_parameters().is_empty() {
            self.generate_send_method(
                &method_name,
                parameters,
                quote! { self.auth },
                quote! { form },
                quote! {
                    let form = reqwest::multipart::Form::new();
                    #form_builder
                },
            )
        } else {
            quote! {
                pub fn #method_name(&self, #(#parameters),*) -> Builder<'_> {
                    let form = reqwest::multipart::Form::new();
                    #form_builder
                    Builder { group: self, form }
                }
            }
        };

        let group_struct_name = self.group.struct_name();
        let method_impl_with_docs = util::add_docs(&self.method.description, method_impl);

        quote! {
            impl<'a> super::#group_struct_name<'a> {
                #method_impl_with_docs
            }
        }
    }

    fn generate_response_struct(&self) -> TokenStream {
        let response = match self.method.types.response() {
            Some(res) => res,
            None => return quote! {},
        };

        let struct_fields = response
            .types
            .iter()
            .map(|field| field.generate_struct_field());

        quote! {
            #[derive(Debug, serde::Deserialize)]
            pub struct Response {
                #(#struct_fields,)*
            }
        }
    }

    /// Returns a TokenStream containing a request builder if there are optional
    /// parameters, otherwise an empty TokenStream is returned.
    fn generate_request_builder(&self) -> TokenStream {
        let optional_params = self.method.types.optional_parameters();
        if optional_params.is_empty() {
            return quote! {};
        }

        let builder_methods = optional_params
            .iter()
            .map(|param| param.generate_optional_builder_method_with_docs());

        let group_name = self.group.struct_name();
        let send_method = self.generate_send_method(
            &util::to_ident("send"),
            vec![],
            quote! { self.group.auth },
            quote! { self.form },
            quote! {},
        );

        quote! {
            pub struct Builder<'a> {
                group: &'a super::#group_name<'a>,
                form: reqwest::multipart::Form,
            }

            impl<'a> Builder<'a> {
                #send_method
                #(#builder_methods)*
            }
        }
    }

    fn generate_send_method(
        &self,
        method_name: &Ident,
        parameters: Vec<TokenStream>,
        auth_access: TokenStream,
        form_access: TokenStream,
        form_factory: TokenStream,
    ) -> TokenStream {
        let method_url = format!("/api/v2/{}/{}", self.group.url, self.method.url);

        let (response_type, response_parse) = match self.method.types.response() {
            Some(resp) => {
                if resp.is_list {
                    (
                        quote! { std::vec::Vec<Response> },
                        quote! { .json::<std::vec::Vec<Response>>() },
                    )
                } else {
                    (quote! { Response }, quote! { .json::<Response>() })
                }
            }
            None => (quote! { String }, quote! { .text() }),
        };

        quote! {
            pub async fn #method_name(self, #(#parameters),*) -> super::super::Result<#response_type> {
                #form_factory
                let res = #auth_access
                    .authenticated_client(#method_url)
                    .multipart(#form_access)
                    .send()
                    .await?
                    #response_parse
                    .await?;

                Ok(res)
            }
        }
    }

    fn mandatory_parameters_as_form_builder(&self) -> TokenStream {
        let builder = self
            .method
            .types
            .mandatory_params()
            .into_iter()
            .map(|param| param.generate_form_builder(quote! { form }));

        quote! {
            #(let #builder)*
        }
    }
}
