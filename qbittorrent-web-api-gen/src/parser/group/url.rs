use regex::Regex;

use crate::{md_parser, parser::util};

impl md_parser::TokenTree {
    pub fn get_group_url(&self) -> String {
        let row = util::find_content_contains(&self.content, "API methods are under")
            .expect("Could not find api method");

        let re = Regex::new(r#"All (?:\w+\s?)+ API methods are under "(\w+)", e.g."#)
            .expect("Failed to create regex");

        let res = re.captures(&row).expect("Failed find capture");
        res[1].to_string()
    }
}
