use crate::md_parser;

use super::{
    api_method::parse_api_method, description::parse_group_description, url_parser::get_group_url,
    ApiGroup,
};

pub fn parse_api_group(tree: md_parser::TokenTree) -> ApiGroup {
    let methods = tree
        .children
        .into_iter()
        .flat_map(parse_api_method)
        .collect();

    let group_description = parse_group_description(&tree.content);
    let group_url = get_group_url(&tree.content);

    let name = tree
        .title
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
