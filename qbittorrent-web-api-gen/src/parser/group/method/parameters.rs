use std::collections::HashMap;

use crate::{md_parser, parser::types};

pub fn parse_parameters(content: &[md_parser::MdContent]) -> Option<Vec<types::Type>> {
    let mut it = content
        .iter()
        .skip_while(|row| match row {
            md_parser::MdContent::Asterisk(content) | md_parser::MdContent::Text(content) => {
                !content.starts_with("Parameters:")
            }
            _ => true,
        })
        // Parameters:              <-- skip
        //                          <-- skip
        // table with parameters    <-- take
        .skip(2);

    let parameter_table = match it.next() {
        Some(md_parser::MdContent::Table(table)) => table,
        _ => return None,
    };

    // empty for now
    let type_map = HashMap::default();

    let table = parameter_table
        .rows
        .iter()
        .flat_map(|row| parse_parameter(row, &type_map))
        .collect();

    Some(table)
}

fn parse_parameter(
    row: &md_parser::TableRow,
    type_map: &HashMap<String, types::TypeDescription>,
) -> Option<types::Type> {
    let description = row.columns.get(2).cloned();

    match &row.columns.get(2) {
        // If the description contains a default value it means that the parameter is optional.
        Some(desc) if desc.contains("default: ") => {
            // type defines a variable as default if it contains: _optional_
            let name_with_optional = format!("{} {}", row.columns[0], types::OPTIONAL);
            types::Type::from(&row.columns[1], &name_with_optional, description, type_map)
        }
        _ => types::Type::from(&row.columns[1], &row.columns[0], description, type_map),
    }
}
