use std::collections::HashMap;

use case::CaseExt;
use quote::{quote, format_ident};
use regex::Regex;

use crate::{generate::util, parser, types};

pub fn create_return_type(
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
