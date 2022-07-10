mod group_parser;
mod object_types;
pub mod types;
mod util;

use group_parser::parse_groups;
use md_parser::{self, TokenTree};
use types::Type;

#[derive(Debug)]
pub struct ApiGroup {
    pub name: String,
    pub methods: Vec<ApiMethod>,
    pub description: Option<String>,
    pub url: String,
}

#[derive(Debug)]
pub struct ApiMethod {
    pub name: String,
    pub description: Option<String>,
    pub parameters: Option<Vec<Type>>,
    pub return_type: Option<ReturnType>,
    pub url: String,
}

#[derive(Debug)]
pub struct ReturnType {
    pub is_list: bool,
    pub parameters: Vec<ReturnTypeParameter>,
}

#[derive(Debug)]
pub struct ReturnTypeParameter {
    pub name: String,
    pub description: String,
    pub return_type: Type,
}

fn extract_relevant_parts(tree: TokenTree) -> Vec<TokenTree> {
    let relevant: Vec<TokenTree> = tree
        .children
        .into_iter()
        .skip_while(|row| match &row.title {
            Some(title) => title != "Authentication",
            None => false,
        })
        .filter(|row| match &row.title {
            Some(title) => title != "WebAPI versioning",
            None => false,
        })
        .collect();

    relevant
}

pub fn parse_api_groups(content: &str) -> Vec<ApiGroup> {
    parse_groups(extract_relevant_parts(md_parser::TokenTreeFactory::create(
        content,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn parse() -> TokenTree {
        let content = include_str!("api-4_1.md");
        let md_tree = md_parser::TokenTreeFactory::create(content);

        let output = format!("{:#?}", md_tree);
        fs::write("token_tree.txt", output).unwrap();

        md_tree
    }

    #[test]
    fn it_works() {
        let groups = parse_groups(extract_relevant_parts(parse()));

        let groups_as_str = format!("{:#?}", groups);
        fs::write("groups.txt", groups_as_str).unwrap();
    }
}
