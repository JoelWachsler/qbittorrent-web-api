mod api_group;
mod api_method;
mod group;
mod group_method;
mod skeleton;
mod util;

use case::CaseExt;
use proc_macro2::TokenStream;
use quote::quote;

use crate::{md_parser, parser};

use self::{group::generate_groups, skeleton::generate_skeleton};

pub fn generate(ast: &syn::DeriveInput, api_content: &str) -> TokenStream {
    let ident = &ast.ident;
    let resp_derives = get_response_derives(ast);

    let token_tree = md_parser::TokenTreeFactory::create(api_content);
    let api_groups = parser::parse_api_groups(token_tree);

    let skeleton = generate_skeleton(ident);
    let groups = generate_groups(api_groups, resp_derives);
    let impl_ident = syn::Ident::new(&format!("{}_impl", ident).to_snake(), ident.span());

    quote! {
        pub mod #impl_ident {
            #skeleton
            #groups
        }
    }
}

fn get_response_derives(ast: &syn::DeriveInput) -> Vec<String> {
    ast.attrs
        .iter()
        .find(|attr| {
            attr.path
                .get_ident()
                .map(|ident| ident == "api_gen")
                .unwrap_or(false)
        })
        .into_iter()
        .flat_map(|attr| attr.parse_meta().ok())
        .filter_map(|meta| match meta {
            syn::Meta::List(list) => Some(list.nested),
            _ => None,
        })
        .flat_map(|nested| nested.into_iter())
        .filter_map(|value| match value {
            syn::NestedMeta::Meta(meta) => Some(meta),
            _ => None,
        })
        .filter_map(|value| match value {
            syn::Meta::NameValue(name_value) => Some(name_value.lit),
            _ => None,
        })
        .filter_map(|lit| match lit {
            syn::Lit::Str(str) => Some(str.value().split(',').map(|s| s.trim()).collect()),
            _ => None,
        })
        .collect()
}
