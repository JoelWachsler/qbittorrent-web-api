use crate::parser;
use case::CaseExt;
use proc_macro2::{Ident, TokenStream};
use quote::quote;

use super::{group_method::GroupMethod, skeleton::auth_ident, util};

#[derive(Debug)]
pub struct GroupGeneration<'a> {
    api_group: parser::ApiGroup,
    struct_derives: &'a [&'a str],
    enum_derives: &'a [&'a str],
}

impl<'a> GroupGeneration<'a> {
    pub fn new(
        api_group: parser::ApiGroup,
        struct_derives: &'a [&'a str],
        enum_derives: &'a [&'a str],
    ) -> Self {
        Self {
            api_group,
            struct_derives,
            enum_derives,
        }
    }

    pub fn generate(&self) -> TokenStream {
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
            self.description(),
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
        self.methods()
            .iter()
            .map(|method| GroupMethod::new(self, method))
            .collect()
    }

    fn description(&self) -> &Option<String> {
        &self.api_group.description
    }

    fn methods(&self) -> &Vec<parser::ApiMethod> {
        &self.api_group.methods
    }

    pub fn url(&self) -> String {
        self.api_group.url.clone()
    }

    pub fn struct_name(&self) -> Ident {
        self.name_camel()
    }

    pub fn struct_derives(&self) -> TokenStream {
        self.derives(self.struct_derives, &[])
    }

    pub fn enum_derives(&self) -> TokenStream {
        self.derives(self.enum_derives, &["PartialEq", "Eq"])
    }

    pub fn derives(&self, derives: &'a [&'a str], additional_derives: &[&str]) -> TokenStream {
        let derives = self
            .all_derives(derives)
            .chain(additional_derives.iter().copied())
            .map(|s| syn::parse_str::<syn::Path>(s).unwrap())
            .map(|derive| quote! { #derive });

        quote! {
            #[derive(#(#derives),*)]
        }
    }

    fn all_derives(&self, derives: &'a [&'a str]) -> impl Iterator<Item = &'a str> {
        let base = vec!["serde::Deserialize", "Debug"].into_iter();
        let additional = derives
            .iter()
            .copied()
            .filter(|item| item != &"serde::Deserialize")
            .filter(|item| item != &"Debug");

        base.chain(additional)
    }

    fn name_camel(&self) -> Ident {
        util::to_ident(&self.api_group.name.to_camel())
    }

    fn name_snake(&self) -> Ident {
        util::to_ident(&self.api_group.name.to_snake())
    }
}
