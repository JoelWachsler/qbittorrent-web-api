use std::rc::Rc;

use super::{md_token::MdContent, token_tree_factory::TokenTreeFactory};

#[derive(Debug)]
pub struct TokenTree {
    pub title: Option<String>,
    pub content: Vec<MdContent>,
    pub children: Vec<TokenTree>,
}

impl From<Rc<TokenTreeFactory>> for TokenTree {
    fn from(builder: Rc<TokenTreeFactory>) -> Self {
        let children = builder
            .children
            .clone()
            .into_inner()
            .into_iter()
            .map(|child| child.into())
            .collect::<Vec<TokenTree>>();

        let content = builder.content.clone().into_inner();

        TokenTree {
            title: builder.title.clone(),
            content,
            children,
        }
    }
}
