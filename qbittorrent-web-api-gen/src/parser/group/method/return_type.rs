use std::collections::HashMap;

use crate::{
    md_parser::{self, MdContent},
    parser::{types, ReturnTypeParameter},
};

#[derive(Debug)]
pub struct ReturnType {
    pub is_list: bool,
    pub parameters: Vec<ReturnTypeParameter>,
}

impl md_parser::TokenTree {
    pub fn parse_return_type(&self) -> Option<ReturnType> {
        let table = self
            .content
            .iter()
            // The response is a ...        <-- Trying to find this line
            //                              <-- The next line is empty
            // Table with the return type   <-- And then extract the following type table
            .skip_while(|row| match row {
                MdContent::Text(text) => !text.starts_with("The response is a"),
                _ => true,
            })
            .find_map(|row| match row {
                MdContent::Table(table) => Some(table),
                _ => None,
            })?;

        let types = self.parse_object_types();

        let parameters = table
            .rows
            .iter()
            .map(|parameter| parameter.to_return_type_parameter(&types))
            .collect();

        Some(ReturnType {
            parameters,
            is_list: self.is_list(),
        })
    }

    fn is_list(&self) -> bool {
        self.content
            .iter()
            .find_map(|row| match row {
                MdContent::Text(text) if text.starts_with("The response is a") => Some(text),
                _ => None,
            })
            .map(|found| found.contains("array"))
            .unwrap_or_else(|| false)
    }

    pub fn parse_object_types(&self) -> HashMap<String, types::TypeDescription> {
        let mut output = HashMap::new();
        let mut content_it = self.content.iter();

        while let Some(entry) = content_it.next() {
            if let md_parser::MdContent::Text(content) = entry {
                const POSSIBLE_VALUES_OF: &str = "Possible values of ";
                if content.contains(POSSIBLE_VALUES_OF) {
                    // is empty
                    content_it.next();
                    if let Some(md_parser::MdContent::Table(table)) = content_it.next() {
                        let name = content
                            .trim_start_matches(POSSIBLE_VALUES_OF)
                            .replace('`', "")
                            .replace(':', "");

                        output.insert(name, table.to_type_description());
                    }
                }
            }
        }

        output
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
        types: &HashMap<String, types::TypeDescription>,
    ) -> ReturnTypeParameter {
        let columns = &self.columns;

        ReturnTypeParameter {
            name: columns[0].clone(),
            description: columns[2].clone(),
            return_type: types::Type::from(
                &columns[1],
                &columns[0],
                Some(columns[2].clone()),
                types,
            )
            .unwrap_or_else(|| panic!("Failed to parse type {}", &columns[1])),
        }
    }
}
