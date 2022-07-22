mod description;
// mod return_type;
mod url;

use crate::{md_parser, types};
use case::CaseExt;
use regex::Regex;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug)]
pub struct ApiMethod {
    pub name: String,
    pub description: Option<String>,
    pub url: String,
    pub types: CompositeTypes,
}

#[derive(Debug)]
pub struct CompositeTypes {
    pub composite_types: Vec<CompositeType>,
}

impl CompositeTypes {
    pub fn new(tables: &Tables) -> Self {
        Self {
            composite_types: tables.get_all_tables_as_types(),
        }
    }

    pub fn parameters(&self) -> Option<&Vec<types::Type>> {
        self.composite_types.iter().find_map(|type_| match type_ {
            CompositeType::Parameters(p) => Some(&p.types),
            _ => None,
        })
    }

    pub fn optional_parameters(&self) -> Option<Vec<&types::Type>> {
        self.parameters()
            .map(|params| params.iter().filter(|param| param.is_optional()).collect())
    }

    pub fn mandatory_params(&self) -> Option<Vec<&types::Type>> {
        self.parameters()
            .map(|params| params.iter().filter(|param| !param.is_optional()).collect())
    }

    pub fn response(&self) -> Option<&Vec<types::Type>> {
        self.composite_types.iter().find_map(|type_| match type_ {
            CompositeType::Response(p) => Some(&p.types),
            _ => None,
        })
    }

    pub fn objects(&self) -> Vec<&TypeWithName> {
        self.composite_types
            .iter()
            .filter_map(|type_| match type_ {
                CompositeType::Object(p) => Some(p),
                _ => None,
            })
            .collect()
    }

    pub fn enums(&self) -> Vec<&Enum> {
        self.composite_types
            .iter()
            .filter_map(|type_| match type_ {
                CompositeType::Enum(p) => Some(p),
                _ => None,
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct ApiParameters {
    pub mandatory: Vec<types::Type>,
    pub optional: Vec<types::Type>,
}

#[derive(Debug)]
pub enum CompositeType {
    Enum(Enum),
    Object(TypeWithName),
    Response(TypeWithoutName),
    Parameters(TypeWithoutName),
}

#[derive(Debug)]
pub struct TypeWithName {
    pub name: String,
    pub types: Vec<types::Type>,
}

#[derive(Debug)]
pub struct TypeWithoutName {
    pub types: Vec<types::Type>,
}

impl TypeWithoutName {
    pub fn new(types: Vec<types::Type>) -> Self {
        Self { types }
    }
}

impl TypeWithName {
    pub fn new(name: &str, types: Vec<types::Type>) -> Self {
        Self {
            name: name.to_string(),
            types,
        }
    }
}

#[derive(Debug)]
pub struct Enum {
    pub name: String,
    pub values: Vec<EnumValue>,
}

#[derive(Debug)]
pub struct EnumValue {
    pub description: Option<String>,
    pub value: String,
    pub original_value: String,
}

impl Enum {
    fn new(name: &str, table: &md_parser::Table) -> Self {
        let values = table.rows.iter().map(EnumValue::from).collect();

        Enum {
            name: name.to_string(),
            values,
        }
    }
}

impl From<&md_parser::TableRow> for EnumValue {
    fn from(row: &md_parser::TableRow) -> Self {
        let description = row.columns.get(1).cloned();
        let original_value = row.columns[0].clone();
        let value = if original_value.parse::<i32>().is_ok() {
            let name = description
                .clone()
                .unwrap()
                .replace(' ', "_")
                .replace('-', "_")
                .replace(',', "_");

            let re = Regex::new(r#"\(.*\)"#).unwrap();
            re.replace_all(&name, "").to_camel()
        } else {
            original_value.to_camel()
        };

        EnumValue {
            description,
            value,
            original_value,
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
        let method_url = child.get_method_url();

        ApiMethod {
            name: name.to_string(),
            description: method_description,
            url: method_url,
            types: CompositeTypes::new(&tables),
        }
    }
}

impl md_parser::TokenTree {
    fn find_content_starts_with(&self, starts_with: &str) -> Option<String> {
        self.content.iter().find_map(|row| match row {
            md_parser::MdContent::Text(content) if content.starts_with(starts_with) => {
                Some(content.into())
            }
            _ => None,
        })
    }
}

impl<'a> From<&'a md_parser::TokenTree> for Tables<'a> {
    fn from(token_tree: &'a md_parser::TokenTree) -> Self {
        let mut tables = BTreeMap::new();
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
pub struct Tables<'a> {
    tables: BTreeMap<String, &'a md_parser::Table>,
}

impl md_parser::Table {
    fn to_enum(&self, input_name: &str) -> Option<CompositeType> {
        let re = Regex::new(r"^Possible values of `(\w+)`$").unwrap();

        if !re.is_match(input_name) {
            return None;
        }

        Some(CompositeType::Enum(Enum::new(
            &Self::regex_to_name(&re, input_name),
            self,
        )))
    }

    fn to_object(&self, input_name: &str) -> Option<CompositeType> {
        let re = Regex::new(r"^(\w+) object$").unwrap();

        if !re.is_match(input_name) {
            return None;
        }

        Some(CompositeType::Object(TypeWithName::new(
            &Self::regex_to_name(&re, input_name),
            self.to_types(),
        )))
    }

    fn to_response(&self, input_name: &str) -> Option<CompositeType> {
        if !input_name.starts_with("The response is a") {
            return None;
        }

        Some(CompositeType::Response(TypeWithoutName::new(
            self.to_types(),
        )))
    }

    fn to_parameters(&self, input_name: &str) -> Option<CompositeType> {
        if !input_name.starts_with("Parameters") {
            return None;
        }

        Some(CompositeType::Parameters(TypeWithoutName::new(
            self.to_types(),
        )))
    }

    fn to_composite_type(&self, input_name: &str) -> Option<CompositeType> {
        self.to_enum(input_name)
            .or_else(|| self.to_response(input_name))
            .or_else(|| self.to_object(input_name))
            .or_else(|| self.to_parameters(input_name))
    }

    fn regex_to_name(re: &Regex, input_name: &str) -> String {
        re.captures(input_name)
            .unwrap()
            .get(1)
            .unwrap()
            .as_str()
            .to_string()
            .to_camel()
    }
}

impl<'a> Tables<'a> {
    fn get_all_tables_as_types(&self) -> Vec<CompositeType> {
        self.tables
            .iter()
            .flat_map(|(k, v)| v.to_composite_type(k))
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
            let tree = TokenTreeFactory::create(input);
            let api_method = ApiMethod::try_new(&tree.children[0]).unwrap();

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

    #[test]
    fn enum_test() {
        run_test!("enum");
    }
}
