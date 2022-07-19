use crate::{md_parser, parser::util};

impl md_parser::TokenTree {
    pub fn get_method_url(&self) -> String {
        const START: &str = "Name: ";

        util::find_content_starts_with(&self.content, START)
            .map(|text| text.trim_start_matches(START).trim_matches('`').to_string())
            .expect("Could find method url")
    }
}
