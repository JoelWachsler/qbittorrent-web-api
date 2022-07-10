use std::collections::HashMap;

use crate::{md_parser::MdContent, parser::types::TypeDescriptions};

use super::types::TypeDescription;

pub fn get_object_types(content: &[MdContent]) -> HashMap<String, TypeDescription> {
    let mut output = HashMap::new();
    let mut content_it = content.iter();
    while let Some(entry) = content_it.next() {
        if let MdContent::Text(content) = entry {
            const POSSIBLE_VALUES_OF: &str = "Possible values of ";
            if content.contains(POSSIBLE_VALUES_OF) {
                // is empty
                content_it.next();
                if let Some(MdContent::Table(table)) = content_it.next() {
                    let enum_types = table
                        .rows
                        .iter()
                        .map(|row| TypeDescriptions {
                            value: row.columns[0].to_string(),
                            description: row.columns[1].to_string(),
                        })
                        .collect();

                    let name = content
                        .trim_start_matches(POSSIBLE_VALUES_OF)
                        .replace('`', "")
                        .replace(':', "");

                    output.insert(name, TypeDescription { values: enum_types });
                }
            }
        }
    }

    output
}
