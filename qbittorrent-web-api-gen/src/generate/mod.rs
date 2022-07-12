mod skeleton;
mod util;

use std::{collections::HashMap, vec::Vec};

use case::CaseExt;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use regex::Regex;

use crate::{md_parser, parser, types};

use self::skeleton::{auth_ident, generate_skeleton};

pub fn generate(ast: &syn::DeriveInput, api_content: &str) -> TokenStream {
    let ident = &ast.ident;

    let token_tree = md_parser::TokenTreeFactory::create(api_content);
    let api_groups = parser::parse_api_groups(token_tree);

    let skeleton = generate_skeleton(ident);
    let groups = generate_groups(api_groups);
    let impl_ident = syn::Ident::new(&format!("{}_impl", ident).to_snake(), ident.span());

    quote! {
        pub mod #impl_ident {
            #skeleton
            #groups
        }
    }
}

pub fn generate_groups(groups: Vec<parser::ApiGroup>) -> proc_macro2::TokenStream {
    let gr = groups
        .iter()
        // implemented manually
        .filter(|group| group.name != "authentication")
        .map(generate_group);

    quote! {
        #(#gr)*
    }
}

fn generate_group(group: &parser::ApiGroup) -> proc_macro2::TokenStream {
    let group_name_camel = util::to_ident(&group.name.to_camel());
    let group_name_snake = util::to_ident(&group.name.to_snake());
    let auth = auth_ident();
    let methods = generate_methods(group, &auth, &group_name_camel);

    let group_method = util::add_docs(
        &group.description,
        quote! {
            pub fn #group_name_snake(&self) -> #group_name_camel {
                #group_name_camel::new(self)
            }
        },
    );

    quote! {
        pub struct #group_name_camel<'a> {
            auth: &'a #auth,
        }

        #methods

        impl #auth {
            #group_method
        }
    }
}

fn generate_methods(
    group: &parser::ApiGroup,
    auth: &syn::Ident,
    group_name_camel: &syn::Ident,
) -> proc_macro2::TokenStream {
    let methods_and_param_structs = group
        .methods
        .iter()
        .map(|method| generate_method(group, method));

    let methods = methods_and_param_structs.clone().map(|(method, ..)| method);
    let structs = methods_and_param_structs.flat_map(|(_, s)| s);

    quote! {
        impl <'a> #group_name_camel<'a> {
            pub fn new(auth: &'a #auth) -> Self {
                Self { auth }
            }

            #(#methods)*
        }

        #(#structs)*
    }
}

fn generate_method(
    group: &parser::ApiGroup,
    method: &parser::ApiMethod,
) -> (proc_macro2::TokenStream, Option<proc_macro2::TokenStream>) {
    let method_name = util::to_ident(&method.name.to_snake());
    let url = format!("/api/v2/{}/{}", group.url, method.url);

    match &method.parameters {
        Some(params) => create_method_with_params(group, method, params, &method_name, &url),
        None => create_method_without_params(group, method, method_name, &url),
    }
}

fn create_method_without_params(
    group: &parser::ApiGroup,
    method: &parser::ApiMethod,
    method_name: proc_macro2::Ident,
    url: &str,
) -> (proc_macro2::TokenStream, Option<proc_macro2::TokenStream>) {
    match create_return_type(group, method) {
        Some((return_type_name, return_type)) => (
            util::add_docs(
                &method.description,
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
            ),
            Some(return_type),
        ),
        None => (
            util::add_docs(
                &method.description,
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
            ), // assume that all methods without a return type returns a string
            None,
        ),
    }
}

fn create_method_with_params(
    group: &parser::ApiGroup,
    method: &parser::ApiMethod,
    params: &[types::Type],
    method_name: &proc_macro2::Ident,
    url: &str,
) -> (proc_macro2::TokenStream, Option<proc_macro2::TokenStream>) {
    let parameter_type = util::to_ident(&format!(
        "{}{}Parameters",
        group.name.to_camel(),
        method.name.to_camel()
    ));

    let mandatory_params = params
        .iter()
        .filter(|param| !param.get_type_info().is_optional);

    let mandatory_param_args = mandatory_params.clone().map(|param| {
        let name = util::to_ident(&param.get_type_info().name.to_snake());
        let t = util::to_ident(&param.to_borrowed_type());

        if param.should_borrow() {
            quote! {
                #name: &#t
            }
        } else {
            quote! {
                #name: #t
            }
        }
    });

    let mandatory_param_names = mandatory_params.clone().map(|param| {
        let name = util::to_ident(&param.get_type_info().name.to_snake());

        quote! {
            #name
        }
    });

    let mandatory_param_args_clone = mandatory_param_args.clone();
    let mandatory_param_form_build = mandatory_params.map(|param| {
        let n = &param.get_type_info().name;
        let name = util::to_ident(&n.to_snake());

        quote! {
            let form = form.text(#n, #name.to_string());
        }
    });

    let optional_params = params
        .iter()
        .filter(|param| param.get_type_info().is_optional)
        .map(|param| {
            let n = &param.get_type_info().name;
            let name = util::to_ident(&n.to_snake());
            let t = util::to_ident(&param.to_borrowed_type());

            let method = if param.should_borrow() {
                quote! {
                    pub fn #name(mut self, value: &#t) -> Self {
                        self.form = self.form.text(#n, value.to_string());
                        self
                    }
                }
            } else {
                quote! {
                    pub fn #name(mut self, value: #t) -> Self {
                        self.form = self.form.text(#n, value.to_string());
                        self
                    }
                }
            };

            util::add_docs(&param.get_type_info().description, method)
        });

    let group_name = util::to_ident(&group.name.to_camel());

    let send = match create_return_type(group, method) {
        Some((return_type_name, return_type)) => {
            quote! {
                impl<'a> #parameter_type<'a> {
                    fn new(group: &'a #group_name, #(#mandatory_param_args),*) -> Self {
                        let form = reqwest::multipart::Form::new();
                        #(#mandatory_param_form_build)*
                        Self { group, form }
                    }

                    #(#optional_params)*

                    pub async fn send(self) -> Result<#return_type_name> {
                        let res = self.group
                            .auth
                            .authenticated_client(#url)
                            .multipart(self.form)
                            .send()
                            .await?
                            .json::<#return_type_name>()
                            .await?;

                        Ok(res)
                    }
                }

                #return_type
            }
        }
        None => {
            quote! {
                impl<'a> #parameter_type<'a> {
                    fn new(group: &'a #group_name, #(#mandatory_param_args),*) -> Self {
                        let form = reqwest::multipart::Form::new();
                        #(#mandatory_param_form_build)*
                        Self { group, form }
                    }

                    #(#optional_params)*

                    pub async fn send(self) -> Result<String> {
                        let res = self.group
                            .auth
                            .authenticated_client(#url)
                            .multipart(self.form)
                            .send()
                            .await?
                            .text()
                            .await?;

                        Ok(res)
                    }
                }
            }
        }
    };

    (
        util::add_docs(
            &method.description,
            quote! {
                pub fn #method_name(&self, #(#mandatory_param_args_clone),*) -> #parameter_type {
                    #parameter_type::new(self, #(#mandatory_param_names),*)
                }
            },
        ),
        Some(quote! {
            pub struct #parameter_type<'a> {
                group: &'a #group_name<'a>,
                form: reqwest::multipart::Form,
            }

            #send
        }),
    )
}

fn create_return_type(
    group: &parser::ApiGroup,
    method: &parser::ApiMethod,
) -> Option<(proc_macro2::TokenStream, proc_macro2::TokenStream)> {
    let return_type = match &method.return_type {
        Some(t) => t,
        None => return None,
    };

    let to_enum_name = |name: &str| {
        format!(
            "{}{}{}",
            group.name.to_camel(),
            method.name.to_camel(),
            name.to_camel()
        )
    };

    let enum_types_with_names =
        return_type
            .parameters
            .iter()
            .flat_map(|parameter| match &parameter.return_type {
                types::Type::Number(types::TypeInfo {
                    ref name,
                    type_description: Some(type_description),
                    ..
                }) => {
                    let enum_fields = type_description.values.iter().map(|value| {
                        let v = &value.value;
                        let re = Regex::new(r#"\(.*\)"#).unwrap();
                        let desc = &value
                            .description
                            .replace(' ', "_")
                            .replace('-', "_")
                            .replace(',', "_");
                        let desc_without_parentheses = re.replace_all(desc, "");
                        let ident = util::to_ident(&desc_without_parentheses.to_camel());

                        util::add_docs(
                            &Some(value.description.clone()),
                            quote! {
                                #[serde(rename = #v)]
                                #ident
                            },
                        )
                    });

                    let enum_name = util::to_ident(&to_enum_name(name));

                    Some((
                        name,
                        quote! {
                            #[allow(clippy::enum_variant_names)]
                            #[derive(Debug, Deserialize, PartialEq, Eq)]
                            pub enum #enum_name {
                                #(#enum_fields,)*
                            }
                        },
                    ))
                }
                types::Type::String(types::TypeInfo {
                    ref name,
                    type_description: Some(type_description),
                    ..
                }) => {
                    let enum_fields = type_description.values.iter().map(|type_description| {
                        let value = &type_description.value;
                        let value_as_ident = util::to_ident(&value.to_camel());

                        util::add_docs(
                            &Some(type_description.description.clone()),
                            quote! {
                                #[serde(rename = #value)]
                                #value_as_ident
                            },
                        )
                    });

                    let enum_name = util::to_ident(&to_enum_name(name));

                    Some((
                        name,
                        quote! {
                            #[allow(clippy::enum_variant_names)]
                            #[derive(Debug, Deserialize, PartialEq, Eq)]
                            pub enum #enum_name {
                                #(#enum_fields,)*
                            }
                        },
                    ))
                }
                _ => None,
            });

    let enum_names: HashMap<&String, String> = enum_types_with_names
        .clone()
        .map(|(enum_name, _)| (enum_name, to_enum_name(enum_name)))
        .collect();

    let enum_types = enum_types_with_names.map(|(_, enum_type)| enum_type);

    let parameters = return_type.parameters.iter().map(|parameter| {
        let namestr = &parameter.name;
        let name = util::to_ident(&namestr.to_snake().replace("__", "_"));
        let rtype = if let Some(enum_type) = enum_names.get(namestr) {
            util::to_ident(enum_type)
        } else {
            util::to_ident(&parameter.return_type.to_owned_type())
        };
        let type_info = parameter.return_type.get_type_info();

        let rtype_as_quote = if type_info.is_list {
            quote! {
                std::vec::Vec<#rtype>
            }
        } else {
            quote! {
                #rtype
            }
        };

        // "type" is a reserved keyword in Rust, so we use a different name.
        if namestr == "type" {
            let non_reserved_name = format_ident!("t_{}", name);
            quote! {
                #[serde(rename = #namestr)]
                pub #non_reserved_name: #rtype_as_quote
            }
        } else {
            quote! {
                #[serde(rename = #namestr)]
                pub #name: #rtype_as_quote
            }
        }
    });

    let return_type_name = util::to_ident(&format!(
        "{}{}Result",
        &group.name.to_camel(),
        &method.name.to_camel()
    ));

    let result_type = if return_type.is_list {
        quote! {
            std::vec::Vec<#return_type_name>
        }
    } else {
        quote! {
            #return_type_name
        }
    };

    Some((
        result_type,
        quote! {
            #[derive(Debug, Deserialize)]
            pub struct #return_type_name {
                #(#parameters,)*
            }

            #(#enum_types)*
        },
    ))
}
