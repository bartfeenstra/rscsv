use std::error::Error;

#[derive(Debug)]
pub struct Format {
    value_delimiter: char,
    value_separator: char,
    escape_character: char,
}

impl Format {
    fn new(value_delimiter: char, value_separator: char, escape_character: char) -> Self {
        Self {
            value_delimiter: value_delimiter,
            value_separator: value_separator,
            escape_character: escape_character,
        }
    }
}

#[derive(Debug)]
struct Cursor {
    row_index: u8,
    char_index: u8,
    escape: bool,
    in_value: bool,
    value_chars: Vec<char>,
    expect: Option<char>,
}

impl Cursor {
    fn new(row_index: u8, format: &Format) -> Self {
        Self {
            row_index: row_index,
            char_index: 0,
            escape: false,
            in_value: false,
            value_chars: vec![],
            // The first expected character is the opening delimiter of the first field.
            expect: Some(format.value_delimiter),
        }
    }
}

#[derive(Debug)]
struct CharLines {
    characters: IntoIter<char>,
    cursor: Cursor,
    format: Format,
}

impl CharLines {
    fn new(characters: IntoIter<char>, format: Format) -> Self {
        Self {
            characters: characters,
            cursor: Cursor::new(0, &format),
            format: format,
        }
    }
}

impl Iterator for CharLines {
    type Item = IntoIter<char>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buffer = vec![];
        while let Some(character) = self.characters.next() {
            // Upon reaching EOL, return the line.
            if character == '\n' {
                let line_characters = buffer.into_iter();
                return Some(line_characters);
            }

            buffer.push(character);
        }

        // We have reached EOF. Return the last of the line, if it isn't empty.
        match buffer.len() {
            0 => None,
            _ => Some(buffer.into_iter()),
        }
    }
}

#[derive(Debug)]
struct CharValues {
    characters: IntoIter<char>,
    cursor: Cursor,
    format: Format,
}

impl CharValues {
    fn new(characters: IntoIter<char>, row_index: u8, format: Format) -> Self {
        CharValues {
            characters: characters,
            cursor: Cursor::new(row_index, &format),
            format: format,
        }
    }
}

impl Iterator for CharValues {
    type Item = Result<String, String>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut value_chars: Vec<char> = vec![];
        while let Some(character) = self.characters.next() {
            // Ignore whitespace outside values.
            if character.is_whitespace() && !self.cursor.in_value {
                continue;
            }

            // Asserts an expected character is present.
            if self.cursor.expect.is_some() {
                let expected = self.cursor.expect.unwrap();
                if character != expected {
                    return Some(Err(format!(
                        "Expected `{}`, but found `{}` on line {}, character {}.",
                        expected, character, self.cursor.row_index, self.cursor.char_index
                    )));
                }
                self.cursor.expect = None;
                // Ignore expected value separators.
                if character == self.format.value_separator {
                    return None;
                }
            }

            // Add escaped characters directly. We cannot be in an escape sequence without being in a value, so we don't have to check for that anymore.
            if self.cursor.escape {
                value_chars.push(character);
                self.cursor.escape = false;
                continue;
            }

            // Toggle escape sequences.
            if character == self.format.escape_character {
                if !self.cursor.in_value {
                    return Some(Err(format!("Encountered an escape character (`{}`) outside value on line {}, character {}.", self.format.escape_character, self.cursor.row_index, self.cursor.char_index)));
                }
                self.cursor.escape = !self.cursor.escape;
                continue;
            }

            // Toggle values.
            if character == self.format.value_delimiter {
                match self.cursor.in_value {
                    true => {
                        self.cursor.expect = Some(self.format.value_separator);
                        self.cursor.in_value = false;
                        return Some(Ok(value_chars.into_iter().collect()));
                    }
                    false => self.cursor.in_value = true,
                };
                continue;
            }

            // Any remaining characters are part of the value.
            value_chars.push(character);
        }
        None
    }
}

// fn parse(characters: Characters, cursor: Cursor, format: Format) -> LineValues {
//     // @todo Map each item to Option<u8>, so we can indicate EOL with None.
//     // @todo Create a first future that processes the first value opening delimiter.
//     // @todo After every character's future chain, execute the resulting future using the next character.
//     // @todo
//     // @todo
//     // @todo
//     // @todo
//     // @todo
//     // @todo
//     // @todo
//     // @todo PROBLEM
//     // @todo PROBLEM
//     // @todo PROBLEM
//     // @todo PROBLEM
//     // @todo PROBLEM
//     // @todo So to create an iterator over a Read impl, that iterator must contain the impl.
//     // @todo Can we do this by making the struct field Read + Sized, or do we need to use a concrete type like File?
//     // @todo Actually, BufReader just needs a Read, which we have.
//     // @todo Then every BufReader iteration corresponds to an iteration here, and we can just map the results.
//     // @todo Inside the mapping, each line would be converted to a Vec<String>
//     // @todo
//     // @todo
//     // @todo
//     // @todo
//     // @todo
// }

// @todo We need a fn that reads a byte and returns a future for it
// @todo Another fns for processing:
// @todo - assert expected character
// @todo - ignore whitespace outside fields
// @todo - assert value delimiters and toggle values
// @todo - toggle escape sequences
// @todo - add characters (escaped and unescaped)
// @todo

// struct Cursor {
//     row_index: u8,
//     mut char_index: u8,
//     mut escape: bool,
//     mut in_value: bool,
//     mut expect: Option<char>,

//     fn new(row_index: u8) {
//         Self {
//             row_index: row_index,
//             char_index: 0,
//             escape: false,
//             in_value: false,
//             mut value_chars: Vec<char> = vec![],
//             // @todo This must be the value delimiter!
//             expect: None,
//         }
//     }
// }

// fn read<'a>(stream: &'a Read) -> impl Future<Item=char, Error=&'a Error> {
//     let mut buffer = [0; 1];
//     while let Ok(count) = stream.read(&mut buffer) {
//         let character = match count {
//             0 => None,
//             _ => Some(buffer[0],)
//         };
//     };
// }

// impl Parser for FileParser {
//     fn parse<T>(&mut self, f: T) -> Result<Vec<Vec<String>>, String>
//     where
//         T: Read + Seek,
//     {
//         let mut fields: Vec<Vec<String>> = vec![];
//         let reader = BufReader::new(f);
//         let mut row_i = 0u8;
//         for line in reader.lines() {
//             row_i = row_i + 1;
//             let mut escape = false;
//             let mut row: Vec<String> = vec![];
//             let mut in_value: bool = false;
//             let mut value_chars: Vec<char> = vec![];
//             // The first expected character is the opening delimiter of the first field.
//             let mut expect: Option<char> = Some(self.value_delimiter);
//             let mut char_i = 0u8;
//             for character in line.unwrap().characters() {
//                 char_i = char_i + 1;
//                 // Ignore whitespace outside values.
//                 if character.is_whitespace() && !in_value {
//                     continue;
//                 }

//                 // Asserts an expected character is present.
//                 if expect.is_some() {
//                     let expected = expect.unwrap();
//                     if character != expected {
//                         return Err(format!(
//                             "Expected `{}`, but found `{}` on line {}, character {}.",
//                             expected, character, row_i, char_i
//                         ));
//                     }
//                     expect = None;
//                     // Ignore expected value separators.
//                     if character == self.value_separator {
//                         continue;
//                     }
//                 }

//                 // Add escaped characters directly. We cannot be in an escape sequence without being in a value, so we don't have to check for that anymore.
//                 if escape {
//                     value_chars.push(character);
//                     escape = false;
//                     continue;
//                 }

//                 // Toggle escape sequences.
//                 if character == self.escape_character {
//                     if !in_value {
//                         return Err(format!("Encountered an escape character (`{}`) outside value on line {}, character {}.", self.escape_character, row_i, char_i));
//                     }
//                     escape = !escape;
//                     continue;
//                 }

//                 // Toggle values.
//                 if character == self.value_delimiter {
//                     match in_value {
//                         true => {
//                             let value: String = value_chars.into_iter().collect();
//                             row.push(value);
//                             value_chars = vec![];
//                             expect = Some(self.value_separator);
//                             in_value = false;
//                         }
//                         false => in_value = true,
//                     };
//                     continue;
//                 }

//                 // Any remaining characters are part of the value.
//                 value_chars.push(character);
//             }
//             fields.push(row);
//         }
//         Ok(fields)
//     }
// }

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn char_lines_should_iterate() {
        let csv_characters = "'foo', 'bar'\n'baz', 'qux'"
            .chars()
            .collect::<Vec<_>>()
            .into_iter();
        let format = Format::new('\'', ',', '\n');
        let lines = CharLines::new(csv_characters, format)
            .map(|characters| characters.collect::<String>())
            .collect::<Vec<_>>();
        assert_eq!(lines, vec!["'foo', 'bar'", "'baz', 'qux'"]);
    }

    #[test]
    fn char_values_should_iterate() {
        let csv_characters = "'foo', 'bar'".chars().collect::<Vec<_>>().into_iter();
        let format = Format::new('\'', ',', '\n');
        let values: Result<Vec<_>, String> = CharValues::new(csv_characters, 0, format)
            .map(|characters| characters.collect::<String>())
            .collect::<Vec<_>>();
        assert_eq!(values.unwrap(), vec!["'foo', 'bar'"]);
    }

    // @todo Test empty lines!!!
    // @todo Test lines with just whitespace!!!

    //     fn assert_file_parser_should_parse(
    //         value_delimiter: character,
    //         value_separator: character,
    //         escape_character: character,
    //         csv: String,
    //         expected: Vec<Vec<&str>>,
    //     ) {
    //         let mut file = tempfile().unwrap();
    //         file.write_fmt(format_args!("{}", csv)).unwrap();
    //         file.flush().unwrap();
    //         file.seek(SeekFrom::Start(0)).unwrap();
    //         let mut parser = FileParser::new(value_delimiter, value_separator, escape_character);
    //         let result = parser.parse(file);
    //         let fields = result.unwrap();
    //         assert_eq!(expected, fields);
    //     }

    //     fn assert_file_parser_should_error(
    //         value_delimiter: character,
    //         value_separator: character,
    //         escape_character: character,
    //         csv: String,
    //     ) {
    //         let mut file = tempfile().unwrap();
    //         file.write_fmt(format_args!("{}", csv)).unwrap();
    //         file.flush().unwrap();
    //         file.seek(SeekFrom::Start(0)).unwrap();
    //         let mut parser = FileParser::new(value_delimiter, value_separator, escape_character);
    //         let result = parser.parse(file);
    //         assert!(result.is_err());
    //     }

    //     #[test]
    //     fn file_parser_should_parse_empty() {
    //         assert_file_parser_should_parse('\'', ',', '\\', "".to_string(), vec![]);
    //     }

    //     #[test]
    //     fn file_parser_should_parse_common() {
    //         assert_file_parser_should_parse(
    //             '\'',
    //             ',',
    //             '\\',
    //             "'foo','bar','baz'".to_string(),
    //             vec![vec!["foo", "bar", "baz"]],
    //         );
    //     }

    //     #[test]
    //     fn file_parser_should_parse_common_multiline() {
    //         assert_file_parser_should_parse(
    //             '\'',
    //             ',',
    //             '\\',
    //             "'foo','bar'\n'baz', 'qux'".to_string(),
    //             vec![vec!["foo", "bar"], vec!["baz", "qux"]],
    //         );
    //     }

    //     #[test]
    //     fn file_parser_should_parse_value_delimiter_hash() {
    //         assert_file_parser_should_parse(
    //             '#',
    //             ',',
    //             '\\',
    //             "#foo#,#bar#,#baz#".to_string(),
    //             vec![vec!["foo", "bar", "baz"]],
    //         );
    //     }

    //     #[test]
    //     fn file_parser_should_parse_value_separator_hash() {
    //         assert_file_parser_should_parse(
    //             '\'',
    //             '#',
    //             '\\',
    //             "'foo'#'bar'#'baz'".to_string(),
    //             vec![vec!["foo", "bar", "baz"]],
    //         );
    //     }

    //     #[test]
    //     fn file_parser_should_parse_escape_sequences_common() {
    //         assert_file_parser_should_parse(
    //             '\'',
    //             ',',
    //             '\\',
    //             "'\\'FOO\\'','\\'BAR\\'','\\'BAZ\\''".to_string(),
    //             vec![vec!["'FOO'", "'BAR'", "'BAZ'"]],
    //         );
    //     }

    //     #[test]
    //     fn file_parser_should_parse_escape_sequences_hash() {
    //         assert_file_parser_should_parse(
    //             '\'',
    //             ',',
    //             '#',
    //             "'#'FOO#'','#'BAR#'','#'BAZ#''".to_string(),
    //             vec![vec!["'FOO'", "'BAR'", "'BAZ'"]],
    //         );
    //     }

    //     #[test]
    //     fn file_parser_should_parse_value_whitespace() {
    //         assert_file_parser_should_parse(
    //             '\'',
    //             ',',
    //             '\\',
    //             "'foo  ', ' bar ',   '   baz'".to_string(),
    //             vec![vec!["foo  ", " bar ", "   baz"]],
    //         );
    //     }

    //     #[test]
    //     fn file_parser_should_parse_surrounding_whitespace() {
    //         assert_file_parser_should_parse(
    //             '\'',
    //             ',',
    //             '\\',
    //             "   'foo  ',     ' bar '         ,   '   baz'      ".to_string(),
    //             vec![vec!["foo  ", " bar ", "   baz"]],
    //         );
    //     }

    //     #[test]
    //     fn file_parser_should_error_on_unopened_first_field() {
    //         assert_file_parser_should_error('\'', ',', '\\', "foo','bar','baz'".to_string());
    //     }

    //     #[test]
    //     fn file_parser_should_error_on_unterminated_first_field() {
    //         assert_file_parser_should_error('\'', ',', '\\', "'foo,'bar','baz'".to_string());
    //     }

    //     #[test]
    //     fn file_parser_should_error_on_invalid_character_before_first_field() {
    //         assert_file_parser_should_error('\'', ',', '\\', "f'oo','bar','baz'".to_string());
    //     }

}
