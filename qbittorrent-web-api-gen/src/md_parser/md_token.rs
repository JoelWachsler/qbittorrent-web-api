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
        let mut output = vec![self.header.raw.clone(), self.split.clone()];
        for row in self.rows.clone() {
            output.push(row.raw);
        }

        output.join("\n")
    }
}

#[derive(Debug, Clone)]
pub struct Header {
    pub level: i32,
    pub content: String,
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

    pub fn from(content: &str) -> Vec<MdToken> {
        // to prevent infinite loops
        let mut max_iterations = 10000;
        let mut decreate_max_iterations = || {
            max_iterations -= 1;
            if max_iterations <= 0 {
                panic!("Max iterations reached, missing termination?");
            };
        };

        let mut output = Vec::new();

        let mut iter = content.lines().peekable();
        while let Some(line) = iter.next() {
            decreate_max_iterations();

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
                while let Some(peeked_row_line) = iter.peek() {
                    decreate_max_iterations();

                    if !peeked_row_line.contains('|') {
                        // we've reached the end of the table, let's go back one step
                        break;
                    }

                    let next_row_line = iter.next().unwrap();
                    let table_row = TableRow {
                        raw: next_row_line.to_string(),
                        columns: to_columns(next_row_line),
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
