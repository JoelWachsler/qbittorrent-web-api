use crate::{md_parser, parser::util};

pub fn get_method_url(content: &[md_parser::MdContent]) -> String {
    const START: &str = "Name: ";

    util::find_content_starts_with(content, START)
        .map(|text| text.trim_start_matches(START).trim_matches('`').to_string())
        .expect("Could find method url")
}
