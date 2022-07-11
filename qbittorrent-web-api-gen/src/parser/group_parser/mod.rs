use crate::md_parser::TokenTree;

use self::{parameters::get_parameters, return_type::get_return_type};

use super::{util, ApiGroup, ApiMethod};

mod description;
mod parameters;
mod return_type;
mod url_parser;

pub fn parse_groups(trees: Vec<TokenTree>) -> Vec<ApiGroup> {
    trees.into_iter().map(parse_api_group).collect()
}

fn parse_api_group(tree: TokenTree) -> ApiGroup {
    let methods = tree
        .children
        .into_iter()
        .flat_map(parse_api_method)
        .collect();

    let group_description = description::get_group_description(&tree.content);
    let group_url = url_parser::get_group_url(&tree.content);

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

fn parse_api_method(child: TokenTree) -> Option<ApiMethod> {
    util::find_content_starts_with(&child.content, "Name: ")
        .map(|name| {
            name.trim_start_matches("Name: ")
                .trim_matches('`')
                .to_string()
        })
        .map(|name| to_api_method(&child, &name))
}

fn to_api_method(child: &TokenTree, name: &str) -> ApiMethod {
    let method_description = description::get_method_description(&child.content);
    let return_type = get_return_type(&child.content);
    let parameters = get_parameters(&child.content);
    let method_url = url_parser::get_method_url(&child.content);

    ApiMethod {
        name: name.to_string(),
        description: method_description,
        parameters,
        return_type,
        url: method_url,
    }
}
