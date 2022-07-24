use crate::{parser, types};
use case::CaseExt;
use proc_macro2::{Ident, TokenStream};
use quote::quote;

use super::{api_group::GroupGeneration, util};

pub fn generate_groups(groups: Vec<parser::ApiGroup>, resp_derives: Vec<String>) -> TokenStream {
    let gr = groups
        .into_iter()
        // implemented manually
        .filter(|group| group.name != "authentication")
        .map(|group| GroupGeneration::new(group, resp_derives.clone()))
        .map(generate_group);

    quote! {
        #(#gr)*
    }
}

fn generate_group(group: GroupGeneration) -> TokenStream {
    let group = group.generate();

    quote! {
        #group
    }
}

#[derive(Debug)]
pub struct StructGenerator<'a> {
    type_: &'a parser::TypeWithName,
    group: &'a GroupGeneration,
}

impl<'a> StructGenerator<'a> {
    pub fn new(type_: &'a parser::TypeWithName, group: &'a GroupGeneration) -> Self {
        Self { type_, group }
    }

    pub fn generate_struct(&self) -> TokenStream {
        let fields = self
            .type_
            .types
            .iter()
            .map(|obj| obj.generate_struct_field());
        let name = util::to_ident(&self.type_.name);
        let derives = self.group.response_derives(vec![]);

        quote! {
            #derives
            pub struct #name {
                #(#fields,)*
            }
        }
    }
}

impl types::Type {
    pub fn generate_struct_field(&self) -> TokenStream {
        let name_snake = self.name_snake();
        let type_ = self.owned_type_ident();
        let orig_name = self.name();

        util::add_docs(
            &self.get_type_info().description,
            quote! {
                #[serde(rename = #orig_name)]
                pub #name_snake: #type_
            },
        )
    }

    fn owned_type_ident(&self) -> TokenStream {
        let owned_type = match self {
            types::Type::Number(_) => quote! { i64 },
            types::Type::Float(_) => quote! { f32 },
            types::Type::Bool(_) => quote! { bool },
            types::Type::String(_) => quote! { String },
            types::Type::StringArray(_) => quote! { String },
            types::Type::Object(obj) => match &obj.ref_type {
                types::RefType::String(str) => {
                    let str_ident = &util::to_ident(str);
                    quote! { #str_ident }
                }
                types::RefType::Map(key, value) => {
                    let key_ident = util::to_ident(key);
                    let value_ident = util::to_ident(value);
                    quote! { std::collections::HashMap<#key_ident, #value_ident> }
                }
            },
        };

        if self.is_list() {
            quote! { std::vec::Vec<#owned_type> }
        } else {
            owned_type
        }
    }

    fn name(&self) -> String {
        self.get_type_info().name.clone()
    }

    fn name_snake(&self) -> Ident {
        util::to_ident(&self.name().to_snake())
    }
}

#[derive(Debug)]
pub struct EnumGeneration<'a> {
    enum_: &'a parser::Enum,
    group: &'a GroupGeneration,
}

impl<'a> EnumGeneration<'a> {
    pub fn new(enum_: &'a parser::Enum, group: &'a GroupGeneration) -> Self {
        Self { enum_, group }
    }

    pub fn generate(&self) -> TokenStream {
        let values = self
            .enum_
            .values
            .iter()
            .map(|enum_value| enum_value.generate());
        let name = util::to_ident(&self.enum_.name);
        let derives = self.group.response_derives(vec!["PartialEq", "Eq"]);

        quote! {
            #[allow(clippy::enum_variant_names)]
            #derives
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

impl types::Type {
    pub fn generate_optional_builder_method_with_docs(&self) -> TokenStream {
        util::add_docs(
            &self.get_type_info().description,
            self.generate_optional_builder_method(),
        )
    }

    fn borrowed_type_ident(&self) -> Ident {
        util::to_ident(&self.to_borrowed_type())
    }

    pub fn to_parameter(&self) -> TokenStream {
        let name_snake = self.name_snake();
        let borrowed_type = self.borrowed_type();

        quote! { #name_snake: #borrowed_type }
    }

    pub fn generate_form_builder(&self, add_to: TokenStream) -> TokenStream {
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
        let type_ = self.borrowed_type_ident();
        if self.should_borrow() {
            quote! { &#type_ }
        } else {
            quote! { #type_ }
        }
    }
}
