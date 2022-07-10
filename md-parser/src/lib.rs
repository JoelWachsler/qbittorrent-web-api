use std::{cell::RefCell, rc::Rc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MdContent {
    Text(String),
    Asterix(String),
    Table(Table),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    pub header: TableRow,
    pub split: String,
    pub rows: Vec<TableRow>,
}

impl Table {
    fn raw(&self) -> String {
        let mut output = Vec::new();
        output.push(self.header.raw.clone());
        output.push(self.split.clone());
        for row in self.rows.clone() {
            output.push(row.raw);
        }

        output.join("\n")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableRow {
    raw: String,
    pub columns: Vec<String>,
}

impl MdContent {
    pub fn inner_value_as_string(&self) -> String {
        match self {
            MdContent::Text(text) => text.into(),
            MdContent::Asterix(text) => text.into(),
            MdContent::Table(table) => table.raw(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Header {
    level: i32,
    content: String,
}

/// These are the only relevant tokens we need for the api generation.
#[derive(Debug)]
pub enum MdToken {
    Header(Header),
    Content(MdContent),
}

impl MdToken {
    fn parse_token(line: &str) -> MdToken {
        if line.starts_with('#') {
            let mut level = 0;
            for char in line.chars() {
                if char != '#' {
                    break;
                }

                level += 1;
            }

            MdToken::Header(Header {
                level,
                content: line.trim_matches('#').trim().to_string(),
            })
        } else if line.starts_with('*') {
            MdToken::Content(MdContent::Asterix(
                line.trim_matches('*').trim().to_string(),
            ))
        } else {
            MdToken::Content(MdContent::Text(line.to_string()))
        }
    }

    fn from(content: &str) -> Vec<MdToken> {
        let mut output = Vec::new();

        let mut iter = content.lines().into_iter();
        while let Some(line) = iter.next() {
            // assume this is a table
            if line.contains('|') {
                let to_columns = |column_line: &str| {
                    column_line
                        .replace('`', "")
                        .split('|')
                        .map(|s| s.trim().to_string())
                        .collect()
                };

                let table_header = TableRow {
                    raw: line.into(),
                    columns: to_columns(line),
                };
                let table_split = iter.next().unwrap();
                let mut table_rows = Vec::new();
                while let Some(row_line) = iter.next() {
                    if !row_line.contains('|') {
                        // we've reached the end of the table, let's go back one step
                        iter.next_back();
                        break;
                    }

                    let table_row = TableRow {
                        raw: row_line.into(),
                        columns: to_columns(row_line),
                    };

                    table_rows.push(table_row);
                }

                output.push(MdToken::Content(MdContent::Table(Table {
                    header: table_header,
                    split: table_split.to_string(),
                    rows: table_rows,
                })));
            } else {
                output.push(MdToken::parse_token(line));
            }
        }

        output
    }
}

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

#[derive(Debug, Default)]
pub struct TokenTreeFactory {
    title: Option<String>,
    content: RefCell<Vec<MdContent>>,
    children: RefCell<Vec<Rc<TokenTreeFactory>>>,
    level: i32,
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
