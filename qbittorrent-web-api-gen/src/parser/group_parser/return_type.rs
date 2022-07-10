use crate::{md_parser::MdContent, parser::{ReturnType, object_types::get_object_types, ReturnTypeParameter, types::Type}};

pub fn get_return_type(content: &[MdContent]) -> Option<ReturnType> {
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

    let types = get_object_types(content);

    let parameters = table
        .rows
        .iter()
        .map(|parameter| ReturnTypeParameter {
            name: parameter.columns[0].clone(),
            description: parameter.columns[2].clone(),
            return_type: Type::from(
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