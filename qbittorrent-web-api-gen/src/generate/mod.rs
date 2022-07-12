mod group;
mod skeleton;
mod util;

use case::CaseExt;
use proc_macro2::TokenStream;
use quote::quote;

use crate::{md_parser, parser};

use self::{group::generate_groups, skeleton::generate_skeleton};

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
