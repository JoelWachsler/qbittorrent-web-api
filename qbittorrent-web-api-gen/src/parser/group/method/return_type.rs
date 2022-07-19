use std::collections::HashMap;

use crate::{
    md_parser,
    parser::{types, ReturnTypeParameter},
};

use super::Tables;

#[derive(Debug)]
pub struct ReturnType {
    pub is_list: bool,
    pub parameters: Vec<ReturnTypeParameter>,
}

impl md_parser::Table {
    fn to_return_type_parameters(
        &self,
        types: &HashMap<String, types::TypeDescription>,
    ) -> Vec<ReturnTypeParameter> {
        self.rows
            .iter()
            .map(|parameter| parameter.to_return_type_parameter(types))
            .collect()
    }
}

impl md_parser::TokenTree {
    pub fn parse_return_type(&self) -> Option<ReturnType> {
        let tables: Tables = self.into();
        let table = tables
            .get_type_containing_as_table("The response is a")
            // these two are special cases not following a pattern
            .or_else(|| tables.get_type_containing_as_table("Possible fields"))
            .or_else(|| {
                tables.get_type_containing_as_table(
                    "Each element of the array has the following properties",
                )
            })?;

        let types = self.parse_object_types();

        Some(ReturnType {
            parameters: table.to_return_type_parameters(&types),
            is_list: self.is_list(),
        })
    }

    fn is_list(&self) -> bool {
        self.find_content_starts_with("The response is a")
            .map(|found| found.contains("array"))
            .unwrap_or_else(|| false)
    }

    pub fn parse_object_types(&self) -> HashMap<String, types::TypeDescription> {
        let tables: Tables = self.into();
        const POSSIBLE_VALUES_OF: &str = "Possible values of ";

        tables
            .get_all_type_containing_as_table(POSSIBLE_VALUES_OF)
            .iter()
            .map(|(k, table)| {
                let name = k
                    .trim_start_matches(POSSIBLE_VALUES_OF)
                    .replace('`', "")
                    .replace(':', "");

                (name, table.to_type_description())
            })
            .collect()
    }
}

impl md_parser::Table {
    pub fn to_type_description(&self) -> types::TypeDescription {
        types::TypeDescription {
            values: self.to_type_descriptions(),
        }
    }

    pub fn to_type_descriptions(&self) -> Vec<types::TypeDescriptions> {
        self.rows
            .iter()
            .map(|row| types::TypeDescriptions {
                value: row.columns[0].to_string(),
                description: row.columns[1].to_string(),
            })
            .collect()
    }
}

impl md_parser::TableRow {
    fn to_return_type_parameter(
        &self,
        type_map: &HashMap<String, types::TypeDescription>,
    ) -> ReturnTypeParameter {
        let columns = &self.columns;

        ReturnTypeParameter {
            name: columns[0].clone(),
            description: columns[2].clone(),
            return_type: self
                .to_types_with_types(type_map)
                .unwrap_or_else(|| panic!("Failed to parse type {}", &columns[1])),
        }
    }
}
