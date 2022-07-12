mod generate;
mod md_parser;
mod parser;
mod util;
mod types;

use proc_macro::TokenStream;
use syn::parse_macro_input;

const API_CONTENT: &str = include_str!("../api-4_1.md");

#[proc_macro_derive(QBittorrentApiGen, attributes(api_gen))]
pub fn derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);
    generate::generate(&ast, API_CONTENT).into()
}
