use regex::Regex;

use crate::md_parser;

impl md_parser::TokenTree {
    pub fn get_group_url(&self) -> String {
        let row = self
            .find_content_contains("API methods are under")
            .expect("Could not find api method");

        let re = Regex::new(r#"All (?:\w+\s?)+ API methods are under "(\w+)", e.g."#)
            .expect("Failed to create regex");

        let res = re.captures(&row).expect("Failed find capture");
        res[1].to_string()
    }

    fn find_content_contains(&self, contains: &str) -> Option<String> {
        self.content.iter().find_map(|row| match row {
            md_parser::MdContent::Text(content) if content.contains(contains) => {
                Some(content.into())
            }
            _ => None,
        })
    }
}
