use crate::md_parser::MdContent;

pub fn find_content_starts_with(content: &[MdContent], starts_with: &str) -> Option<String> {
    content.iter().find_map(|row| match row {
        MdContent::Text(content) => {
            if content.starts_with(starts_with) {
                Some(content.into())
            } else {
                None
            }
        }
        _ => None,
    })
}

pub fn find_content_contains(content: &[MdContent], contains: &str) -> Option<String> {
    content.iter().find_map(|row| match row {
        MdContent::Text(content) => {
            if content.contains(contains) {
                Some(content.into())
            } else {
                None
            }
        }
        _ => None,
    })
}
