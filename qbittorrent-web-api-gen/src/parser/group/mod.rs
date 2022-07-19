mod description;
mod method;
mod url;

use crate::md_parser;

pub use method::*;

#[derive(Debug)]
pub struct ApiGroup {
    pub name: String,
    pub methods: Vec<ApiMethod>,
    pub description: Option<String>,
    pub url: String,
}

impl ApiGroup {
    pub fn new(tree: &md_parser::TokenTree) -> ApiGroup {
        ApiGroup {
            name: tree.name(),
            methods: tree.methods(),
            description: tree.parse_group_description(),
            url: tree.get_group_url(),
        }
    }
}

impl md_parser::TokenTree {
    fn name(&self) -> String {
        self.title
            .clone()
            .unwrap()
            .to_lowercase()
            .trim_end_matches("(experimental)")
            .trim()
            .replace(' ', "_")
    }

    fn methods(&self) -> Vec<ApiMethod> {
        self.children.iter().flat_map(ApiMethod::try_new).collect()
    }
}
