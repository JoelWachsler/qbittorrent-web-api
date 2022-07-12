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

pub fn parse_return_type(content: &[MdContent]) -> Option<ReturnType> {
    let table = content
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

    let types = parse_object_types(content);

    let parameters = table
        .rows
        .iter()
        .map(|parameter| ReturnTypeParameter {
            name: parameter.columns[0].clone(),
            description: parameter.columns[2].clone(),
            return_type: types::Type::from(
                &parameter.columns[1],
                &parameter.columns[0],
                Some(parameter.columns[2].clone()),
                &types,
            )
            .unwrap_or_else(|| panic!("Failed to parse type {}", &parameter.columns[1])),
        })
        .collect();

    let is_list = content
        .iter()
        .find_map(|row| match row {
            MdContent::Text(text) if text.starts_with("The response is a") => Some(text),
            _ => None,
        })
        .map(|found| found.contains("array"))
        .unwrap_or_else(|| false);

    Some(ReturnType {
        parameters,
        is_list,
    })
}

pub fn parse_object_types(
    content: &[md_parser::MdContent],
) -> HashMap<String, types::TypeDescription> {
    let mut output = HashMap::new();
    let mut content_it = content.iter();

    while let Some(entry) = content_it.next() {
        if let md_parser::MdContent::Text(content) = entry {
            const POSSIBLE_VALUES_OF: &str = "Possible values of ";
            if content.contains(POSSIBLE_VALUES_OF) {
                // is empty
                content_it.next();
                if let Some(md_parser::MdContent::Table(table)) = content_it.next() {
                    let enum_types = to_type_descriptions(table);

                    let name = content
                        .trim_start_matches(POSSIBLE_VALUES_OF)
                        .replace('`', "")
                        .replace(':', "");

                    output.insert(name, types::TypeDescription { values: enum_types });
                }
            }
        }
    }

    output
}

fn to_type_descriptions(table: &md_parser::Table) -> Vec<types::TypeDescriptions> {
    table
        .rows
        .iter()
        .map(|row| types::TypeDescriptions {
            value: row.columns[0].to_string(),
            description: row.columns[1].to_string(),
        })
        .collect()
}
