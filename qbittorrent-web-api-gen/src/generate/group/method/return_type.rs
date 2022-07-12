use std::collections::HashMap;

use case::CaseExt;
use quote::{format_ident, quote};
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

    let enum_types_with_names = return_type
        .parameters
        .iter()
        .flat_map(|parameter| match &parameter.return_type {
            types::Type::Number(types::TypeInfo {
                ref name,
                type_description: Some(type_description),
                ..
            }) => create_enum_field_value(type_description, name, create_number_enum_value),
            types::Type::String(types::TypeInfo {
                ref name,
                type_description: Some(type_description),
                ..
            }) => create_enum_field_value(type_description, name, create_string_enum_value),
            _ => None,
        })
        .flat_map(|(name, enum_fields)| {
            let enum_name = util::to_ident(&to_enum_name(&name));

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
        });

    let enum_names: HashMap<String, String> = enum_types_with_names
        .clone()
        .map(|(enum_name, _)| (enum_name.clone(), to_enum_name(&enum_name)))
        .collect();

    let enum_types = enum_types_with_names.map(|(_, enum_type)| enum_type);

    let builder_fields = return_type.parameters.iter().map(|parameter| {
        let namestr = &parameter.name;
        let name = util::to_ident(&namestr.to_snake().replace("__", "_"));
        let enum_name = match enum_names.get(namestr) {
            Some(enum_type) => enum_type.to_owned(),
            None => parameter.return_type.to_owned_type(),
        };
        let rtype = util::to_ident(&enum_name);
        let rtype_as_quote = if parameter.return_type.get_type_info().is_list {
            quote! { std::vec::Vec<#rtype> }
        } else {
            quote! { #rtype }
        };

        let generate_field = |field_name| {
            quote! {
                #[serde(rename = #namestr)]
                pub #field_name: #rtype_as_quote
            }
        };

        // "type" is a reserved keyword in Rust, so we just add "t_" to it.
        if namestr == "type" {
            generate_field(format_ident!("t_{}", name))
        } else {
            generate_field(name)
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
                #(#builder_fields,)*
            }

            #(#enum_types)*
        },
    ))
}

fn create_enum_field_value<F>(
    type_description: &types::TypeDescription,
    name: &str,
    f: F,
) -> Option<(String, Vec<proc_macro2::TokenStream>)>
where
    F: Fn(&types::TypeDescriptions) -> proc_macro2::TokenStream,
{
    let enum_fields: Vec<proc_macro2::TokenStream> = type_description
        .values
        .iter()
        .map(f)
        .collect::<Vec<proc_macro2::TokenStream>>();

    let nn = name.to_string();

    Some((nn, enum_fields))
}

fn create_string_enum_value(
    type_description: &types::TypeDescriptions,
) -> proc_macro2::TokenStream {
    let value = &type_description.value;
    let value_as_ident = util::to_ident(&value.to_camel());
    create_enum_field(&value_as_ident, value, &type_description.description)
}

fn create_number_enum_value(value: &types::TypeDescriptions) -> proc_macro2::TokenStream {
    let v = &value.value;
    let re = Regex::new(r#"\(.*\)"#).unwrap();
    let desc = &value
        .description
        .replace(' ', "_")
        .replace('-', "_")
        .replace(',', "_");
    let desc_without_parentheses = re.replace_all(desc, "");
    let ident = util::to_ident(&desc_without_parentheses.to_camel());

    create_enum_field(&ident, v, &value.description)
}

fn create_enum_field(
    ident: &syn::Ident,
    rename: &str,
    description: &str,
) -> proc_macro2::TokenStream {
    util::add_docs(
        &Some(description.to_string()),
        quote! {
            #[serde(rename = #rename)]
            #ident
        },
    )
}
