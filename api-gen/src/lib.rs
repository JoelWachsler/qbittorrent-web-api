mod group;
mod skeleton;
mod util;

use case::CaseExt;
use proc_macro::TokenStream;
use quote::quote;
use skeleton::generate_skeleton;
use syn::parse_macro_input;

use crate::group::generate_groups;

const API_CONTENT: &str = include_str!("api-4_1.md");

#[proc_macro_derive(QBittorrentApiGen, attributes(api_gen))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);
    let ident = &ast.ident;

    let api_groups = parser::parse_api_groups(API_CONTENT);

    let skeleton = generate_skeleton(ident);
    let groups = generate_groups(api_groups);
    let impl_ident = syn::Ident::new(&format!("{}_impl", ident).to_snake(), ident.span());

    quote! {
        mod #impl_ident {
            #skeleton
            #groups
        }
    }
    .into()
}
