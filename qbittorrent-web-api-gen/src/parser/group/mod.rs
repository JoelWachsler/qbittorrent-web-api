mod description;
mod method;
mod url;

use crate::md_parser;

use self::{description::parse_group_description, method::parse_api_method, url::get_group_url};
pub use method::{ApiMethod, ReturnType};

#[derive(Debug)]
pub struct ApiGroup {
    pub name: String,
    pub methods: Vec<ApiMethod>,
    pub description: Option<String>,
    pub url: String,
}

pub fn parse_api_group(tree: &md_parser::TokenTree) -> ApiGroup {
    let methods = tree.children.iter().flat_map(parse_api_method).collect();

    let group_description = parse_group_description(&tree.content);
    let group_url = get_group_url(&tree.content);

    let name = tree
        .title
        .clone()
        .unwrap()
        .to_lowercase()
        .trim_end_matches("(experimental)")
        .trim()
        .replace(' ', "_");

    ApiGroup {
        name,
        methods,
        description: group_description,
        url: group_url,
    }
}
