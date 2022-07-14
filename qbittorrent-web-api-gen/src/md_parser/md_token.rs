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
        let mut output = Vec::new();

        let mut iter = content.lines();
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
