use regex::Regex;

use crate::{md_parser, parser::util};

pub fn get_group_url(content: &[md_parser::MdContent]) -> String {
    let row = util::find_content_contains(content, "API methods are under")
        .expect("Could not find api method");

    let re = Regex::new(r#"All (?:\w+\s?)+ API methods are under "(\w+)", e.g."#)
        .expect("Failed to create regex");

    let res = re.captures(&row).expect("Failed find capture");
    res[1].to_string()
}
