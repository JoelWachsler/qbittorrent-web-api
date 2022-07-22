use crate::{parser, types};
use case::CaseExt;
use proc_macro2::{Ident, TokenStream};
use quote::quote;

use super::{skeleton::auth_ident, util};

pub fn generate_groups(groups: Vec<parser::ApiGroup>) -> TokenStream {
    let gr = groups
        .iter()
        // implemented manually
        .filter(|group| group.name != "authentication")
        .map(generate_group);

    quote! {
        #(#gr)*
    }
}

fn generate_group(group: &parser::ApiGroup) -> TokenStream {
    let group = group.generate();

    quote! {
        #group
    }
}

impl parser::ApiGroup {
    fn generate(&self) -> TokenStream {
        let struct_name = self.struct_name();
        let group_name_snake = self.name_snake();
        let group_methods = self.generate_group_methods();

        let group_struct = self.group_struct();
        let group_factory = self.group_factory();
        let auth = auth_ident();

        quote! {
            pub mod #group_name_snake {
                impl <'a> #struct_name<'a> {
                    pub fn new(auth: &'a super::#auth) -> Self {
                        Self { auth }
                    }
                }

                #group_struct
                #group_factory

                #(#group_methods)*
            }
        }
    }

    fn generate_group_methods(&self) -> Vec<TokenStream> {
        let group_methods = self.group_methods();
        group_methods
            .iter()
            .map(|group_method| group_method.generate_method())
            .collect()
    }

    fn group_factory(&self) -> TokenStream {
        let struct_name = self.struct_name();
        let name_snake = self.name_snake();
        let auth = auth_ident();

        util::add_docs(
            &self.description,
            quote! {
                impl super::#auth {
                    pub fn #name_snake(&self) -> #struct_name {
                        #struct_name::new(self)
                    }
                }
            },
        )
    }

    fn group_struct(&self) -> TokenStream {
        let struct_name = self.struct_name();
        let auth = auth_ident();

        quote! {
            #[derive(Debug)]
            pub struct #struct_name<'a> {
                auth: &'a super::#auth,
            }
        }
    }

    fn group_methods(&self) -> Vec<GroupMethod> {
        self.methods
            .iter()
            .map(|method| GroupMethod::new(self, method))
            .collect()
    }

    fn struct_name(&self) -> Ident {
        self.name_camel()
    }

    fn name_camel(&self) -> Ident {
        util::to_ident(&self.name.to_camel())
    }

    fn name_snake(&self) -> Ident {
        util::to_ident(&self.name.to_snake())
    }
}

impl parser::ApiMethod {
    fn structs(&self) -> TokenStream {
        let objects = self.types.objects();
        let structs = objects.iter().map(|obj| obj.generate_struct());

        quote! {
            #(#structs)*
        }
    }

    fn enums(&self) -> TokenStream {
        let enums = self.types.enums();
        let generated_enums = enums.iter().map(|e| e.generate());

        quote! {
            #(#generated_enums)*
        }
    }

    fn name_snake(&self) -> Ident {
        util::to_ident(&self.name.to_snake())
    }
}

impl parser::TypeWithName {
    fn generate_struct(&self) -> TokenStream {
        let fields = self.types.iter().map(|obj| obj.generate_struct_field());
        let name = util::to_ident(&self.name);

        quote! {
            #[derive(Debug, serde::Deserialize)]
            pub struct #name {
                #(#fields,)*
            }
        }
    }
}

impl types::Type {
    fn generate_struct_field(&self) -> TokenStream {
        let name_snake = self.name_snake();
        let type_name = util::to_ident(&self.to_owned_type());
        let type_ = if self.is_list() {
            quote! { std::vec::Vec<#type_name> }
        } else {
            quote! { #type_name }
        };
        let orig_name = self.name();

        util::add_docs(
            &self.get_type_info().description,
            quote! {
                #[serde(rename = #orig_name)]
                pub #name_snake: #type_
            },
        )
    }

    fn name(&self) -> String {
        self.get_type_info().name.clone()
    }

    fn name_snake(&self) -> Ident {
        util::to_ident(&self.name().to_snake())
    }
}

impl parser::Enum {
    fn generate(&self) -> TokenStream {
        let values = self.values.iter().map(|enum_value| enum_value.generate());
        let name = util::to_ident(&self.name);

        quote! {
            #[allow(clippy::enum_variant_names)]
            #[derive(Debug, serde::Deserialize, PartialEq, Eq)]
            pub enum #name {
                #(#values,)*
            }
        }
    }
}

impl parser::EnumValue {
    fn generate(&self) -> TokenStream {
        util::add_docs(&self.description, self.generate_field())
    }

    fn generate_field(&self) -> TokenStream {
        let orig_name = self.original_value.clone();

        // special enum value which does not follow conventions
        if orig_name == "\"/path/to/download/to\"" {
            quote! {
                PathToDownloadTo(String)
            }
        } else {
            let name_camel = self.name_camel();
            quote! {
                #[serde(rename = #orig_name)]
                #name_camel
            }
        }
    }

    fn name_camel(&self) -> Ident {
        util::to_ident(&self.value.to_camel())
    }
}

#[derive(Debug)]
struct GroupMethod<'a> {
    group: &'a parser::ApiGroup,
    method: &'a parser::ApiMethod,
}

impl<'a> GroupMethod<'a> {
    fn new(group: &'a parser::ApiGroup, method: &'a parser::ApiMethod) -> Self {
        Self { group, method }
    }

    fn generate_method(&self) -> TokenStream {
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

impl types::Type {
    fn generate_optional_builder_method_with_docs(&self) -> TokenStream {
        util::add_docs(
            &self.get_type_info().description,
            self.generate_optional_builder_method(),
        )
    }

    fn borrowed_type_ident(&self) -> Ident {
        util::to_ident(&self.to_borrowed_type())
    }

    fn to_parameter(&self) -> TokenStream {
        let name_snake = self.name_snake();
        let borrowed_type = self.borrowed_type();

        quote! { #name_snake: #borrowed_type }
    }

    fn generate_form_builder(&self, add_to: TokenStream) -> TokenStream {
        let name_str = self.name();
        let name_snake = self.name_snake();

        quote! {
            #add_to = #add_to.text(#name_str, #name_snake.to_string());
        }
    }

    fn generate_optional_builder_method(&self) -> TokenStream {
        let name_snake = self.name_snake();
        let borrowed_type = self.borrowed_type();
        let form_builder = self.generate_form_builder(quote! { self.form });

        quote! {
            pub fn #name_snake(mut self, #name_snake: #borrowed_type) -> Self {
                #form_builder;
                self
            }
        }
    }

    fn borrowed_type(&self) -> TokenStream {
        if self.should_borrow() {
            let type_ = self.borrowed_type_ident();
            quote! { &#type_ }
        } else {
            let type_ = self.borrowed_type_ident();
            quote! { #type_ }
        }
    }
}
