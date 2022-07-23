use crate::parser;
use case::CaseExt;
use proc_macro2::{Ident, TokenStream};
use quote::quote;

use super::{group_method::GroupMethod, skeleton::auth_ident, util};

impl parser::ApiGroup {
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

    pub fn struct_name(&self) -> Ident {
        self.name_camel()
    }

    fn name_camel(&self) -> Ident {
        util::to_ident(&self.name.to_camel())
    }

    fn name_snake(&self) -> Ident {
        util::to_ident(&self.name.to_snake())
    }
}
