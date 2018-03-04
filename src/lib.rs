use std::fs::File;

pub trait Parser {
    fn parse(f: File) -> Option<Vec<Vec<String>>>;
}

pub struct FileParser {
    value_delimiter: String,
    value_separator: String,
    line_separator: String,
}

impl FileParser {
    fn new(value_delimiter: String, value_separator: String, line_separator: String) -> Self {
        Self {
            value_delimiter: value_delimiter,
            value_separator: value_separator,
            line_separator: line_separator,
        }
    }
}

impl Parser for FileParser {
    fn parse(f: File) -> Option<Vec<Vec<String>>> {
        None
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn file_parser_should_new() {
        FileParser::new("'".into(), ",".into(), "\n".into());
    }

}
