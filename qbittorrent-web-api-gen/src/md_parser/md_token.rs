#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MdContent {
    Text(String),
    Asterisk(String),
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
            MdContent::Asterisk(text) => text.into(),
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
    pub fn from(content: &str) -> Vec<MdToken> {
        // to prevent infinite loops
        let mut max_iterator_checker = MaxIteratorChecker::default();

        let mut output = Vec::new();
        let mut iter = content.lines().peekable();

        while let Some(line) = iter.next() {
            max_iterator_checker.decrease();

            if line.contains(" | ") || line.contains("-|") || line.contains("|-") {
                let table = TableParser::new(&mut max_iterator_checker, &mut iter).parse(line);
                output.push(MdToken::Content(table));
            } else if line.starts_with('#') {
                output.push(parse_header(line));
            } else if line.starts_with('*') {
                let asterisk = MdContent::Asterisk(line.trim_matches('*').trim().to_string());
                output.push(MdToken::Content(asterisk));
            } else {
                output.push(MdToken::Content(MdContent::Text(line.to_string())));
            }
        }

        output
    }
}

fn parse_header(line: &str) -> MdToken {
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
}

struct TableParser<'a, 'b> {
    max_iterator_checker: &'a mut MaxIteratorChecker,
    iter: &'a mut std::iter::Peekable<std::str::Lines<'b>>,
}

impl<'a, 'b> TableParser<'a, 'b> {
    fn new(
        max_iterator_checker: &'a mut MaxIteratorChecker,
        iter: &'a mut std::iter::Peekable<std::str::Lines<'b>>,
    ) -> Self {
        Self {
            max_iterator_checker,
            iter,
        }
    }

    fn parse(&mut self, line: &str) -> MdContent {
        let table_header = TableRow {
            raw: line.into(),
            columns: Self::to_columns(line),
        };

        let table_split = self.iter.next().unwrap();
        let table_rows = self.table_rows();

        MdContent::Table(Table {
            header: table_header,
            split: table_split.to_string(),
            rows: table_rows,
        })
    }

    fn table_rows(&mut self) -> Vec<TableRow> {
        let mut table_rows = Vec::new();

        while let Some(peeked_row_line) = self.iter.peek() {
            self.max_iterator_checker.decrease();

            if !peeked_row_line.contains('|') {
                // we've reached the end of the table, let's go back one step
                break;
            }

            let next_row_line = self.iter.next().unwrap();
            table_rows.push(TableRow {
                raw: next_row_line.to_string(),
                columns: Self::to_columns(next_row_line),
            });
        }

        table_rows
    }

    fn to_columns(column_line: &str) -> Vec<String> {
        column_line
            .replace('`', "")
            .split('|')
            .map(|s| s.trim().to_string())
            .collect()
    }
}

#[derive(Debug)]
struct MaxIteratorChecker {
    max_iterations: i32,
}

impl MaxIteratorChecker {
    fn decrease(&mut self) {
        self.max_iterations -= 1;
        if self.max_iterations <= 0 {
            panic!("Max iterations reached, missing termination?");
        }
    }
}

impl Default for MaxIteratorChecker {
    fn default() -> Self {
        MaxIteratorChecker {
            max_iterations: 10000,
        }
    }
}
