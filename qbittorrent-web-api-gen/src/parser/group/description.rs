use crate::md_parser;

pub fn parse_group_description(content: &[md_parser::MdContent]) -> Option<String> {
    let return_desc = content
        .iter()
        .map(|row| row.inner_value_as_string())
        .collect::<Vec<String>>()
        .join("\n")
        .trim()
        .to_string();

    if return_desc.is_empty() {
        None
    } else {
        Some(return_desc)
    }
}
