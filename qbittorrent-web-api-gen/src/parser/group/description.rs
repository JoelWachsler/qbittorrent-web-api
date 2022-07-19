use crate::md_parser;

impl md_parser::TokenTree {
    pub fn parse_group_description(&self) -> Option<String> {
        let return_desc = self
            .content
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
}
