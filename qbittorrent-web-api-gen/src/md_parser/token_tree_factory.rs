use std::{cell::RefCell, rc::Rc};

use super::{
    md_token::{Header, MdContent, MdToken},
    token_tree::TokenTree,
};

#[derive(Debug, Default)]
pub struct TokenTreeFactory {
    pub title: Option<String>,
    pub content: RefCell<Vec<MdContent>>,
    pub children: RefCell<Vec<Rc<TokenTreeFactory>>>,
    pub level: i32,
}

impl TokenTreeFactory {
    fn new(title: &str, level: i32) -> Self {
        Self {
            title: if title.is_empty() {
                None
            } else {
                Some(title.to_string())
            },
            level,
            ..Default::default()
        }
    }

    fn add_content(&self, content: MdContent) {
        self.content.borrow_mut().push(content);
    }

    fn append(&self, child: &Rc<TokenTreeFactory>) {
        self.children.borrow_mut().push(child.clone());
    }

    pub fn create(content: &str) -> TokenTree {
        let tokens = MdToken::from(content);

        let mut stack = Vec::new();
        let root = Rc::new(TokenTreeFactory::default());
        stack.push(root.clone());

        for token in tokens {
            match token {
                MdToken::Header(Header { level, content }) => {
                    let new_header = Rc::new(TokenTreeFactory::new(&content, level));

                    // go back until we're at the same or lower level.
                    while let Some(current) = stack.pop() {
                        if current.level < level {
                            current.append(&new_header);
                            stack.push(current);
                            break;
                        }
                    }

                    stack.push(new_header.clone());
                }
                MdToken::Content(content) => {
                    let current = stack.pop().unwrap();
                    current.add_content(content);
                    stack.push(current);
                }
            }
        }

        root.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! TEST_DIR {
        () => {
            "token_tree_factory_tests"
        };
    }

    macro_rules! run_test {
        ($test_file:expr) => {
            use pretty_assertions::assert_eq;

            // given
            let input = include_str!(concat!(TEST_DIR!(), "/", $test_file, ".md"));

            // when
            let tree = TokenTreeFactory::create(input);

            // then
            let tree_as_str = format!("{tree:#?}");
            let should_be = include_str!(concat!(TEST_DIR!(), "/", $test_file, ".check"));
            assert_eq!(tree_as_str, should_be);
        };
    }

    // use this macro when creating/updating as test
    #[allow(unused_macros)]
    macro_rules! update_test {
        ($test_file:expr) => {
            use std::fs;
            use std::path::Path;

            let input = include_str!(concat!(TEST_DIR!(), "/", $test_file, ".md"));
            let tree = TokenTreeFactory::create(input);
            let tree_as_str = format!("{tree:#?}");
            let file = concat!("src/md_parser/", TEST_DIR!(), "/", $test_file, ".check");

            // prevent user from accidentally leaving the current macro in a test
            if Path::new(file).exists() {
                panic!("Test case already exists: {file}");
            }

            fs::write(file, tree_as_str).unwrap();
        };
    }

    #[test]
    fn should_remove_surrounding_asterisk() {
        run_test!("should_remove_surrounding_asterisk");
    }

    #[test]
    fn should_remove_surrounding_hash() {
        run_test!("should_remove_surrounding_hash");
    }

    #[test]
    fn single_level() {
        run_test!("single_level");
    }

    #[test]
    fn complex() {
        run_test!("complex");
    }

    #[test]
    fn log() {
        run_test!("log");
    }

    #[test]
    fn multi_table() {
        run_test!("multi_table");
    }

    #[test]
    fn non_table_with_pipe() {
        run_test!("non_table_with_pipe");
    }
}
