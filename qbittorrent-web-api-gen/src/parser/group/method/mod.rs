mod description;
mod return_type;
mod url;

use crate::{md_parser, parser::util, types};
pub use return_type::ReturnType;
use std::collections::HashMap;

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

impl ApiMethod {
    pub fn try_new(child: &md_parser::TokenTree) -> Option<Self> {
        const NAME: &str = "Name: ";

        child
            .find_content_starts_with(NAME)
            .map(|name| name.trim_start_matches(NAME).trim_matches('`').to_string())
            .map(|name| ApiMethod::new(child, &name))
    }

    fn new(child: &md_parser::TokenTree, name: &str) -> Self {
        let tables = Tables::from(child);
        let method_description = child.parse_method_description();
        let return_type = child.parse_return_type();
        // let return_type = tables.return_type().map(|r| ReturnType::new(r));
        let parameters = tables
            .get_type_containing("Parameters")
            .map(ApiParameters::new);
        let method_url = child.get_method_url();

        ApiMethod {
            name: name.to_string(),
            description: method_description,
            parameters,
            return_type,
            url: method_url,
        }
    }
}

impl md_parser::TokenTree {
    fn find_content_starts_with(&self, content: &str) -> Option<String> {
        util::find_content_starts_with(&self.content, content)
    }
}

impl<'a> From<&'a md_parser::TokenTree> for Tables<'a> {
    fn from(token_tree: &'a md_parser::TokenTree) -> Self {
        let mut tables = HashMap::new();
        let mut prev_prev: Option<&md_parser::MdContent> = None;
        let mut prev: Option<&md_parser::MdContent> = None;

        for content in &token_tree.content {
            if let md_parser::MdContent::Table(table) = content {
                let title = match prev_prev {
                    Some(md_parser::MdContent::Text(text)) => text.clone(),
                    Some(md_parser::MdContent::Asterisk(text)) => text.clone(),
                    _ => panic!("Expected table title, found: {:?}", prev_prev),
                };

                tables.insert(title.replace(':', ""), table);
            }

            prev_prev = prev;
            prev = Some(content);
        }

        Tables { tables }
    }
}

#[derive(Debug)]
struct Tables<'a> {
    tables: HashMap<String, &'a md_parser::Table>,
}

impl<'a> Tables<'a> {
    fn get_type_containing(&self, name: &str) -> Option<Vec<types::Type>> {
        self.get_type_containing_as_table(name)
            .map(|table| table.to_types())
    }

    fn get_type_containing_as_table(&self, name: &str) -> Option<&md_parser::Table> {
        self.get_all_type_containing_as_table(name)
            .iter()
            .map(|(_, table)| *table)
            .find(|_| true)
    }

    fn get_all_type_containing_as_table(&self, name: &str) -> HashMap<String, &md_parser::Table> {
        self.tables
            .iter()
            .filter(|(key, _)| key.contains(name))
            .map(|(k, table)| (k.clone(), *table))
            .collect()
    }
}

impl md_parser::Table {
    fn to_types(&self) -> Vec<types::Type> {
        self.rows
            .iter()
            .flat_map(|table_row| table_row.to_type())
            .collect()
    }
}

impl md_parser::TableRow {
    fn to_type(&self) -> Option<types::Type> {
        self.to_types_with_types(&HashMap::new())
    }

    fn to_types_with_types(
        &self,
        type_map: &HashMap<String, types::TypeDescription>,
    ) -> Option<types::Type> {
        let columns = &self.columns;
        let description = columns.get(2).cloned();

        match &columns.get(2) {
            // If the description contains a default value it means that the parameter is optional.
            Some(desc) if desc.contains("default: ") => {
                // type defines a variable as default if it contains: _optional_
                let name_with_optional = format!("{} {}", columns[0], types::OPTIONAL);
                types::Type::from(&columns[1], &name_with_optional, description, type_map)
            }
            _ => types::Type::from(&columns[1], &columns[0], description, type_map),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use md_parser::TokenTreeFactory;

    macro_rules! TEST_DIR {
        () => {
            "method_tests"
        };
    }

    #[allow(unused_macros)]
    macro_rules! run_test {
        ($test_file:expr) => {
            use pretty_assertions::assert_eq;

            // given
            let input = include_str!(concat!(TEST_DIR!(), "/", $test_file, ".md"));

            // when
            let tree = TokenTreeFactory::create(input);
            let api_method = ApiMethod::try_new(&tree.children[0]).unwrap();

            // then
            let api_method_as_str = format!("{api_method:#?}");
            let should_be = include_str!(concat!(TEST_DIR!(), "/", $test_file, ".check"));
            assert_eq!(api_method_as_str, should_be);
        };
    }

    // use this macro when creating/updating as test
    #[allow(unused_macros)]
    macro_rules! update_test {
        ($test_file:expr) => {
            use std::fs;
            use std::path::Path;

            let input = include_str!(concat!(TEST_DIR!(), "/", $test_file, ".md"));
            let tree = ApiMethod::try_new(input);
            let api_method = parse_api_method(&tree.children[0]).unwrap();

            let tree_as_str = format!("{tree:#?}");
            let api_method_as_str = format!("{api_method:#?}");

            let tree_file = concat!(
                "src/parser/group/method/",
                TEST_DIR!(),
                "/",
                $test_file,
                ".tree"
            );
            let file = concat!(
                "src/parser/group/method/",
                TEST_DIR!(),
                "/",
                $test_file,
                ".check"
            );

            // prevent user from accidentally leaving the current macro in a test
            if Path::new(file).exists() {
                panic!("Test case already exists: {file}");
            }

            fs::write(file, api_method_as_str).unwrap();
            fs::write(tree_file, tree_as_str).unwrap();
        };
    }

    #[test]
    fn search_result() {
        run_test!("search_result");
    }
}
