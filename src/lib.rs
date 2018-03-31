extern crate tempfile;

use std::error::Error;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::fs::File;
use tempfile::tempfile;

pub trait Parser {
    fn parse<T>(&mut self, f: T) -> Result<Vec<Vec<String>>, String>
    where
        T: Read + Seek;
}

pub struct FileParser {
    value_delimiter: char,
    value_separator: char,
    escape_character: char,
}

impl FileParser {
    fn new(value_delimiter: char, value_separator: char, escape_character: char) -> Self {
        Self {
            value_delimiter: value_delimiter,
            value_separator: value_separator,
            escape_character: escape_character,
        }
    }
}

impl Parser for FileParser {
    fn parse<T>(&mut self, f: T) -> Result<Vec<Vec<String>>, String>
    where
        T: Read + Seek,
    {
        let mut fields: Vec<Vec<String>> = vec![];
        let reader = BufReader::new(f);
        let mut row_i = 0u8;
        for line in reader.lines() {
            row_i = row_i + 1;
            let mut escape = false;
            let mut row: Vec<String> = vec![];
            let mut in_value: bool = false;
            let mut value_chars: Vec<char> = vec![];
            // The first expected character is the opening delimiter of the first field.
            let mut expect: Option<char> = Some(self.value_delimiter);
            let mut char_i = 0u8;
            for char in line.unwrap().chars() {
                char_i = char_i + 1;
                // Ignore whitespace outside values.
                if char.is_whitespace() && !in_value {
                    continue;
                }

                // Asserts an expected character is present.
                if expect.is_some() {
                    let expected = expect.unwrap();
                    if char != expected {
                        return Err(format!(
                            "Expected `{}`, but found `{}` on line {}, character {}.",
                            expected, char, row_i, char_i
                        ));
                    }
                    expect = None;
                    // Ignore expected value separators.
                    if char == self.value_separator {
                        continue;
                    }
                }

                // Add escaped characters directly. We cannot be in an escape sequence without being in a value, so we don't have to check for that anymore.
                if escape {
                    value_chars.push(char);
                    escape = false;
                    continue;
                }

                // Toggle escape sequences.
                if char == self.escape_character {
                    if !in_value {
                        return Err(format!("Encountered an escape character (`{}`) outside value on line {}, character {}.", self.escape_character, row_i, char_i));
                    }
                    escape = !escape;
                    continue;
                }

                // Toggle values.
                if char == self.value_delimiter {
                    match in_value {
                        true => {
                            let value: String = value_chars.into_iter().collect();
                            row.push(value);
                            value_chars = vec![];
                            expect = Some(self.value_separator);
                            in_value = false;
                        }
                        false => in_value = true,
                    };
                    continue;
                }

                // Any remaining characters are part of the value.
                value_chars.push(char);
            }
            fields.push(row);
        }
        Ok(fields)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn assert_file_parser_should_parse(
        value_delimiter: char,
        value_separator: char,
        escape_character: char,
        csv: String,
        expected: Vec<Vec<&str>>,
    ) {
        let mut file = tempfile().unwrap();
        file.write_fmt(format_args!("{}", csv)).unwrap();
        file.flush().unwrap();
        file.seek(SeekFrom::Start(0)).unwrap();
        let mut parser = FileParser::new(value_delimiter, value_separator, escape_character);
        let result = parser.parse(file);
        let fields = result.unwrap();
        assert_eq!(expected, fields);
    }

    fn assert_file_parser_should_error(
        value_delimiter: char,
        value_separator: char,
        escape_character: char,
        csv: String,
    ) {
        let mut file = tempfile().unwrap();
        file.write_fmt(format_args!("{}", csv)).unwrap();
        file.flush().unwrap();
        file.seek(SeekFrom::Start(0)).unwrap();
        let mut parser = FileParser::new(value_delimiter, value_separator, escape_character);
        let result = parser.parse(file);
        assert!(result.is_err());
    }

    #[test]
    fn file_parser_should_parse_empty() {
        assert_file_parser_should_parse('\'', ',', '\\', "".to_string(), vec![]);
    }

    #[test]
    fn file_parser_should_parse_common() {
        assert_file_parser_should_parse(
            '\'',
            ',',
            '\\',
            "'foo','bar','baz'".to_string(),
            vec![vec!["foo", "bar", "baz"]],
        );
    }

    #[test]
    fn file_parser_should_parse_common_multiline() {
        assert_file_parser_should_parse(
            '\'',
            ',',
            '\\',
            "'foo','bar'\n'baz', 'qux'".to_string(),
            vec![vec!["foo", "bar"], vec!["baz", "qux"]],
        );
    }

    #[test]
    fn file_parser_should_parse_value_delimiter_hash() {
        assert_file_parser_should_parse(
            '#',
            ',',
            '\\',
            "#foo#,#bar#,#baz#".to_string(),
            vec![vec!["foo", "bar", "baz"]],
        );
    }

    #[test]
    fn file_parser_should_parse_value_separator_hash() {
        assert_file_parser_should_parse(
            '\'',
            '#',
            '\\',
            "'foo'#'bar'#'baz'".to_string(),
            vec![vec!["foo", "bar", "baz"]],
        );
    }

    #[test]
    fn file_parser_should_parse_escape_sequences_common() {
        assert_file_parser_should_parse(
            '\'',
            ',',
            '\\',
            "'\\'FOO\\'','\\'BAR\\'','\\'BAZ\\''".to_string(),
            vec![vec!["'FOO'", "'BAR'", "'BAZ'"]],
        );
    }

    #[test]
    fn file_parser_should_parse_escape_sequences_hash() {
        assert_file_parser_should_parse(
            '\'',
            ',',
            '#',
            "'#'FOO#'','#'BAR#'','#'BAZ#''".to_string(),
            vec![vec!["'FOO'", "'BAR'", "'BAZ'"]],
        );
    }

    #[test]
    fn file_parser_should_parse_value_whitespace() {
        assert_file_parser_should_parse(
            '\'',
            ',',
            '\\',
            "'foo  ', ' bar ',   '   baz'".to_string(),
            vec![vec!["foo  ", " bar ", "   baz"]],
        );
    }

    #[test]
    fn file_parser_should_parse_surrounding_whitespace() {
        assert_file_parser_should_parse(
            '\'',
            ',',
            '\\',
            "   'foo  ',     ' bar '         ,   '   baz'      ".to_string(),
            vec![vec!["foo  ", " bar ", "   baz"]],
        );
    }

    #[test]
    fn file_parser_should_error_on_unopened_first_field() {
        assert_file_parser_should_error('\'', ',', '\\', "foo','bar','baz'".to_string());
    }

    #[test]
    fn file_parser_should_error_on_unterminated_first_field() {
        assert_file_parser_should_error('\'', ',', '\\', "'foo,'bar','baz'".to_string());
    }

    #[test]
    fn file_parser_should_error_on_invalid_character_before_first_field() {
        assert_file_parser_should_error('\'', ',', '\\', "f'oo','bar','baz'".to_string());
    }

}
