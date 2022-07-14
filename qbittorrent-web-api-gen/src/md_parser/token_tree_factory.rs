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

    #[test]
    fn should_remove_surrounding_asterix() {
        // given
        let input = r#"
# A
**B**
        "#
        .trim_matches('\n')
        .trim();

        // when
        let tree = TokenTreeFactory::create(input);

        // then
        println!("{:#?}", tree);
        let first = tree.children.first().unwrap();
        let content = first.content.first().unwrap();
        assert_eq!(*content, MdContent::Asterix("B".into()));
    }

    #[test]
    fn should_remove_surrounding_hash() {
        // given
        let input = r#"
# A #
        "#
        .trim_matches('\n')
        .trim();

        // when
        let tree = TokenTreeFactory::create(input);

        // then
        println!("{:#?}", tree);
        assert_eq!(tree.children.first().unwrap().title, Some("A".into()));
    }

    #[test]
    fn single_level() {
        // given
        let input = r#"
# A
Foo
        "#
        .trim_matches('\n')
        .trim();

        // when
        let tree = TokenTreeFactory::create(input);

        // then
        println!("{:#?}", tree);
        assert_eq!(tree.title, None);
        let first_child = tree.children.first().unwrap();
        assert_eq!(first_child.title, Some("A".into()));
    }

    #[test]
    fn complex() {
        // given
        let input = r#"
# A
Foo
## B
# C
## D
Bar
        "#
        .trim_matches('\n')
        .trim();

        // when
        let tree = TokenTreeFactory::create(input);

        // then
        println!("{:#?}", tree);
        assert_eq!(tree.title, None);
        assert_eq!(tree.children.len(), 2);

        let first = tree.children.get(0).unwrap();
        assert_eq!(first.title, Some("A".into()));
        assert_eq!(first.children.len(), 1);
        assert_eq!(first.children.first().unwrap().title, Some("B".into()));

        let second = tree.children.get(1).unwrap();
        assert_eq!(second.title, Some("C".into()));
        assert_eq!(second.children.len(), 1);
        assert_eq!(second.children.first().unwrap().title, Some("D".into()));
    }
}
