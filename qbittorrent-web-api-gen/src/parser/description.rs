use crate::md_parser::MdContent;

pub fn get_group_description(content: &[MdContent]) -> Option<String> {
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

pub fn get_method_description(content: &[MdContent]) -> Option<String> {
    let return_desc = content
        .iter()
        // skip until we get to the "Returns:" text
        .skip_while(|row| match row {
            MdContent::Asterix(text) => !text.starts_with("Returns:"),
            _ => true,
        })
        // there is one space before the table
        .skip(2)
        .skip_while(|row| match row {
            MdContent::Text(text) => !text.is_empty(),
            _ => true,
        })
        // and there is one space after the table
        .skip(1)
        // then what is left should be the description
        .flat_map(|row| match row {
            MdContent::Text(text) => Some(text),
            _ => None,
        })
        .cloned()
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
