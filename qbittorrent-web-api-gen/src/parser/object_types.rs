use std::collections::HashMap;

use crate::{md_parser, parser::types};

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
