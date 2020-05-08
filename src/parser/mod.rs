use std::{error,fmt};
use std::num::ParseFloatError;
use std::vec::Vec;
use std::collections::BTreeMap;


pub mod metric_parser;
pub mod service_check_parser;

#[derive(Debug,PartialEq)]
pub enum ParseError {
    /// No content in statsd message
    EmptyInput,
    /// Incomplete input in statsd message
    IncompleteInput,
    /// No name in input
    NoName,
    /// Value is not a float
    ValueNotFloat,
    /// Sample rate is not a float
    SampleRateNotFloat,
    /// Metric type is unknown
    UnknownMetricType,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ParseError::EmptyInput => write!(f, "Empty input"),
            ParseError::IncompleteInput => write!(f, "Incomplete input"),
            ParseError::NoName => write!(f, "No name in input"),
            ParseError::ValueNotFloat => write!(f, "Value is not a float"),
            ParseError::SampleRateNotFloat => write!(f, "Sample rate is not a float"),
            ParseError::UnknownMetricType => write!(f, "Unknown metric type")
        }
    }
}

impl error::Error for ParseError {
  // Implement description so that older versions of rust still work
  fn description(&self) -> &str {
    "description() is deprecated; use Display"
  }
}

#[derive(Debug,PartialEq)]
pub struct Parser {
    chars: Vec<char>,
    len: usize,
    pos: usize
}

impl Parser {
    // Returns a Parser for given string
    pub fn new(buf: String) -> Parser {
        let chars: Vec<char> = buf.trim_end().chars().collect();
        let len = chars.len();
        Parser {
            chars: chars,
            len:   len,
            pos:   0
        }
    }

    /// Consumes the buffer until the given character is found
    /// or the end is reached
    fn take_until(&mut self, to_match: Vec<char>) -> String {
        let mut chars = Vec::new();
        loop {
            if self.pos >= self.len {
                break
            }
            let current_char = self.chars[self.pos];
            self.pos += 1;
            if to_match.contains(&current_char) {
                break
            } else {
                chars.push(current_char);
            }
        }
        chars.into_iter().collect()
    }

    /// Consumes the buffer untill the character is found
    /// or the end is reached, the result is parsed into a float
    fn take_float_until(&mut self, to_match: Vec<char>) -> Result<f64, ParseFloatError> {
        let string = self.take_until(to_match);
        string.parse()
    }

    /// Returns the current character in the buffer
    fn peek(&mut self) -> Option<char> {
        if self.pos == self.len {
            None
        } else {
            Some(self.chars[self.pos])
        }
    }

    /// Returns the previous character in the buffer
    fn last(&mut self) -> Option<char> {
        if self.pos == 0 {
            None
        } else {
            Some(self.chars[self.pos - 1 ])
        }
    }

    /// Moves the buffer to the next position
    fn skip(&mut self) {
        self.pos += 1;
    }

    fn parse_tags(&mut self) -> BTreeMap<String, String> {
        let mut tags = BTreeMap::new();

        self.skip(); // Skip the `#`

        // Loop over the remaining buffer and see
        // if we can find key/value pairs, separated by : and ,
        // in the format key:value,key:value
        loop {
            // Stop the loop if we've encountered a separator (|)
            if Some('|') == self.last() {
              break
            }

            // Stop the loop if we have nothing left to parse
            let tag = self.take_until(vec![',', '|']);
            if tag.is_empty() {
                break
            }

            // Split the string on ':' and use the first part as key, last parts as value
            // host:localhost:3000 will become key: host, value: localhost:3000
            let mut split= tag.split(":");
            match split.next() {
                Some(key) => {
                  let parts: Vec<&str> = split.collect();
                  tags.insert(key.to_owned(), parts.join(":"))
                },
                None => break
            };
        }

        tags
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::Parser;

    #[test]
    fn test_take_until() {
        let mut parser = Parser::new("this is a string".to_string());

        // Returns up untill the first occurrence of the character
        assert_eq!(parser.take_until(vec![' ']), "this");

        // Moves the position to the first occurrence
        assert_eq!(parser.pos, 5);

        // Returns the rest of the string if character is not found
        assert_eq!(parser.take_until(vec!['.']), "is a string");

        // Moves the position to the end of the string
        assert_eq!(parser.pos, 16);
    }

    #[test]
    fn test_take_float_until() {
        let mut parser = Parser::new("10.01|number|string".to_string());

        // Returns float up untill the first occurrence of the character
        assert_eq!(parser.take_float_until(vec!['|']), Ok(10.01));

        // Moves the position to the first occurrence
        assert_eq!(parser.pos, 6);

        // Returns err if not float
        assert!(parser.take_float_until(vec!['|']).is_err());

        // Moves the position to the end of the string
        assert_eq!(parser.pos, 13);
    }

    #[test]
    fn test_peek() {
        let mut parser = Parser::new("this is a string".to_string());
        parser.pos = 10;

        // Returns the character at the current position
        assert_eq!(parser.peek(), Some('s'));

        // It does not move the position
        assert_eq!(parser.pos, 10);

        parser.pos = 16;

        // Returns None if we're at the end of the string
        assert_eq!(parser.peek(), None);
    }

    #[test]
    fn test_last() {
        let mut parser = Parser::new("abcdef".to_string());
        parser.pos = 0;

        // Returns None if we're at the beginning
        assert_eq!(parser.last(), None);

        // It does not move the position
        assert_eq!(parser.pos, 0);

        parser.pos = 3;

        // Returns the character if we're not at the beginning
        assert_eq!(parser.last(), Some('c'));
    }

    #[test]
    fn test_skip() {
        let mut parser = Parser::new("foo#bar".to_string());
        parser.pos = 3;
        parser.skip();

        // Increases the position by one
        assert_eq!(parser.pos, 4);
    }

    #[test]
    fn test_parse_tags() {
        let mut parser = Parser::new("#hostname:frontend1,redis_instance:10.0.0.16:6379,namespace:web".to_string());

        let mut tags = BTreeMap::new();
        tags.insert("hostname".to_string(), "frontend1".to_string());
        tags.insert("redis_instance".to_string(), "10.0.0.16:6379".to_string());
        tags.insert("namespace".to_string(), "web".to_string());

        // Increases the position by one
        assert_eq!(parser.parse_tags(), tags);
    }
}
