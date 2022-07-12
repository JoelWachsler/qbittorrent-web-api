use crate::md_parser;

use super::{
    description::parse_method_description, parameters::parse_parameters,
    return_type::parse_return_type, url_parser::get_method_url, util, ApiMethod,
};

pub fn parse_api_method(child: md_parser::TokenTree) -> Option<ApiMethod> {
    util::find_content_starts_with(&child.content, "Name: ")
        .map(|name| {
            name.trim_start_matches("Name: ")
                .trim_matches('`')
                .to_string()
        })
        .map(|name| to_api_method(&child, &name))
}

fn to_api_method(child: &md_parser::TokenTree, name: &str) -> ApiMethod {
    let method_description = parse_method_description(&child.content);
    let return_type = parse_return_type(&child.content);
    let parameters = parse_parameters(&child.content);
    let method_url = get_method_url(&child.content);

    ApiMethod {
        name: name.to_string(),
        description: method_description,
        parameters,
        return_type,
        url: method_url,
    }
}
