mod description;
mod parameters;
mod return_type;
mod url;

use crate::{md_parser, parser::util, types};

pub use return_type::ReturnType;

use self::{
    description::parse_method_description, parameters::parse_parameters,
    return_type::parse_return_type, url::get_method_url,
};

#[derive(Debug)]
pub struct ApiMethod {
    pub name: String,
    pub description: Option<String>,
    pub parameters: Option<ApiParameters>,
    pub return_type: Option<ReturnType>,
    pub url: String,
}

#[derive(Debug)]
pub struct ApiParameters {
    pub mandatory: Vec<types::Type>,
    pub optional: Vec<types::Type>,
}

impl ApiParameters {
    fn new(params: Vec<types::Type>) -> Self {
        let (mandatory, optional) = params.into_iter().fold(
            (vec![], vec![]),
            |(mut mandatory, mut optional), parameter| {
                if parameter.get_type_info().is_optional {
                    optional.push(parameter);
                } else {
                    mandatory.push(parameter);
                }

                (mandatory, optional)
            },
        );

        Self {
            mandatory,
            optional,
        }
    }
}

pub fn parse_api_method(child: &md_parser::TokenTree) -> Option<ApiMethod> {
    util::find_content_starts_with(&child.content, "Name: ")
        .map(|name| {
            name.trim_start_matches("Name: ")
                .trim_matches('`')
                .to_string()
        })
        .map(|name| to_api_method(child, &name))
}

fn to_api_method(child: &md_parser::TokenTree, name: &str) -> ApiMethod {
    let method_description = parse_method_description(&child.content);
    let return_type = parse_return_type(&child.content);
    let parameters = parse_parameters(&child.content).map(ApiParameters::new);
    let method_url = get_method_url(&child.content);

    ApiMethod {
        name: name.to_string(),
        description: method_description,
        parameters,
        return_type,
        url: method_url,
    }
}
