use std::{error,fmt};
use std::num::ParseFloatError;
use std::vec::Vec;
use std::collections::HashMap;

use {MetricType,Metric};

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
    UnknownMetricType
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

impl error::Error for ParseError {}

#[derive(Debug,PartialEq)]
pub struct Parser {
    chars: Vec<char>,
    len: usize,
    pos: usize
}

impl Parser {
    // Returns a Parser for given string
    pub fn new(buf: String) -> Parser {
        let chars: Vec<char> = buf.chars().collect();
        let len = chars.len();
        Parser {
            chars: chars,
            len:   len,
            pos:   0
        }
    }

    /// Consumes the buffer until the given character is found
    /// or the end is reached
    fn take_until(&mut self, to_match: char) -> String {
        let mut chars = Vec::new();
        loop {
            if self.pos >= self.len {
                break
            }
            let current_char = self.chars[self.pos];
            self.pos += 1;
            if to_match == current_char {
                break
            } else {
                chars.push(current_char);
            }
        }
        chars.into_iter().collect()
    }

    /// Consumes the buffer untill the character is found
    /// or the end is reached, the result is parsed into a float
    fn take_float_until(&mut self, to_match: char) -> Result<f64, ParseFloatError> {
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

    /// Moves the buffer to the next position
    fn skip(&mut self) {
        self.pos += 1;
    }

    /// Runs the parser, returns a Metric struct
    pub fn parse(mut self) -> Result<Metric, ParseError> {
        if self.chars.is_empty() {
            return Err(ParseError::EmptyInput)
        }

        // Start with the name
        let name = self.take_until(':');

        if name.is_empty() {
            return Err(ParseError::NoName)
        }

        // The value should be everything until the first pipe (`|`)
        let value = match self.take_float_until('|') {
            Ok(v) => v,
            Err(_) => return Err(ParseError::ValueNotFloat)
        };

        // The metric type should be everything until the next pipe, or the end
        let metric_type = match self.take_until('|').as_ref() {
            "ms"  => MetricType::Timing,
            "c"   => MetricType::Counter,
            "g"   => MetricType::Gauge,
            "m"   => MetricType::Meter,
            "h"   => MetricType::Histogram,
            _other => return Err(ParseError::UnknownMetricType)
        };

        // The next part can either be the sample rate or tags,
        // peek the value and match on `@` to get the sample rate
        let sample_rate = match self.peek() {
            Some('@') => {
                self.skip(); // Skip the `@`
                match self.take_float_until('|') {
                    Ok(v) => Some(v),
                    Err(_) => return Err(ParseError::SampleRateNotFloat)
                }
            }
            _ => None
        };

        // Peek the remaining string, if it starts with a pound (`#`)
        // try and match tags
        let tags = if Some('#') == self.peek() {
            let mut tags = HashMap::new();

            self.skip(); // Skip the `#`

            // Loop over the remaining buffer and see
            // if we can find key/value pairs, separated by : and ,
            // in the format key:value,key:value
            loop {
                let key = self.take_until(':');
                let val = self.take_until(',');

                // If we have both a key and a value, add it to the tags
                // otherwise stop the loop
                if key.len() > 0 && val.len() > 0 {
                    tags.insert(key, val);
                } else {
                    break
                }
            }

            Some(tags)
        } else {
            None
        };

        Ok(Metric {
            name: name,
            value: value,
            metric_type: metric_type,
            sample_rate: sample_rate,
            tags: tags
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::Parser;
    use {Metric,MetricType};

    #[test]
    fn test_take_until() {
        let mut parser = Parser::new("this is a string".to_string());

        // Returns up untill the first occurrence of the character
        assert_eq!(parser.take_until(' '), "this");

        // Moves the position to the first occurrence
        assert_eq!(parser.pos, 5);

        // Returns the rest of the string if character is not found
        assert_eq!(parser.take_until('.'), "is a string");

        // Moves the position to the end of the string
        assert_eq!(parser.pos, 16);
    }

    #[test]
    fn test_take_float_until() {
        let mut parser = Parser::new("10.01|number|string".to_string());

        // Returns float up untill the first occurrence of the character
        assert_eq!(parser.take_float_until('|'), Ok(10.01));

        // Moves the position to the first occurrence
        assert_eq!(parser.pos, 6);

        // Returns err if not float
        assert!(parser.take_float_until('|').is_err());

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
    fn skip() {
        let mut parser = Parser::new("foo#bar".to_string());
        parser.pos = 3;
        parser.skip();

        // Increases the position by one
        assert_eq!(parser.pos, 4);
    }

    #[test]
    fn test_parse_with_tags() {
        let parser = Parser::new("service.duration:101|ms|@0.9|#hostname:frontend1,namespace:web".to_string());

        let mut tags = HashMap::new();
        tags.insert("hostname".to_string(), "frontend1".to_string());
        tags.insert("namespace".to_string(), "web".to_string());

        let expected = Metric {
            name: "service.duration".to_string(),
            value: 101.0,
            metric_type: MetricType::Timing,
            sample_rate: Some(0.9),
            tags: Some(tags)
        };

        assert_eq!(parser.parse(), Ok(expected));
    }

    #[test]
    fn test_parse_without_tags() {
        let parser = Parser::new("service.duration:101|ms|@0.9|".to_string());

        let expected = Metric {
            name: "service.duration".to_string(),
            value: 101.0,
            metric_type: MetricType::Timing,
            sample_rate: Some(0.9),
            tags: None
        };

        assert_eq!(parser.parse(), Ok(expected));
    }

    #[test]
    fn test_parse_invalid() {
        let parser = Parser::new("service.duration:101|aaa|@0.9|".to_string());
        assert!(parser.parse().is_err());
    }

    // Details of all supported statsd messages are test in lib.rs
}
