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
    let struct_derives = get_derives(ast, "struct_derives");
    let enum_derives = get_derives(ast, "enum_derives");

    let token_tree = md_parser::TokenTreeFactory::create(api_content);
    let api_groups = parser::parse_api_groups(token_tree);

    let skeleton = generate_skeleton(ident);
    let groups = generate_groups(api_groups, struct_derives, enum_derives);
    let impl_ident = syn::Ident::new(&format!("{}_impl", ident).to_snake(), ident.span());

    quote! {
        pub mod #impl_ident {
            #skeleton
            #groups
        }
    }
}

fn get_derives(ast: &syn::DeriveInput, name: &str) -> Vec<String> {
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
            syn::Meta::NameValue(name_value) => Some(name_value),
            _ => None,
        })
        .filter(|name_value| {
            name_value
                .path
                .segments
                .first()
                .map(|seg| seg.ident == name)
                .unwrap_or(false)
        })
        .map(|name_value| name_value.lit)
        .filter_map(|lit| match lit {
            syn::Lit::Str(str) => Some(str.value().split(',').map(|s| s.trim()).collect()),
            _ => None,
        })
        .collect()
}
