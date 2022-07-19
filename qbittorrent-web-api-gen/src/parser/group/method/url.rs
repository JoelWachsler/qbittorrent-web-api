use crate::md_parser;

impl md_parser::TokenTree {
    pub fn get_method_url(&self) -> String {
        const START: &str = "Name: ";

        self.find_content_starts_with(START)
            .map(|text| text.trim_start_matches(START).trim_matches('`').to_string())
            .expect("Could find method url")
    }
}
