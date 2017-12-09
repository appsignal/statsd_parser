use std::vec::Vec;
use std::collections::HashMap;

use {MetricType, ParseResult};

#[derive(Debug,PartialEq)]
pub struct Parser {
    buf: String,
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
            buf:   buf,
            chars: chars,
            len:   len,
            pos:   0
        }
    }

    /// Consumes the buffer until the given character is found
    // or the end is reached
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
    /// if that fails, the default is returned
    fn take_float_until(&mut self, to_match: char, default: f64) -> f64 {
        let string = self.take_until(to_match);
        match string.parse() {
            Ok(res) => res,
            Err(_) => default
        }
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

    /// Runs the parser, returns a ParseResult struct
    pub fn parse(mut self) -> ParseResult {
        let mut tags = HashMap::new();

        // Start with the name
        let name = self.take_until(':');

        // The value should be everything until the first pipe (`|`)
        let value = self.take_float_until('|', 0.0);

        // The metric type should be everything until the next pipe, or the end
        let metric_type = match self.take_until('|').as_ref() {
            "ms" => MetricType::Timing,
            "c"  => MetricType::Counter,
            "g"  => MetricType::Gauge,
            other => panic!(format!("Could not get metric type from {:}", other))
        };

        // The next part can either be the sample rate or tags,
        // peek the value and match on `@` to get the sample rate
        let sample_rate = match self.peek() {
            Some('@') => {
                self.skip(); // Skip the `@`
                self.take_float_until('|', 0.0)
            }
            _ => 0.0
        };

        // Peek the remaining string, if it starts with a pound (`#`)
        // try and match tags
        if Some('#') == self.peek() {
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
                    break;
                }
            }
        };

        return ParseResult {
            name: name,
            value: value,
            metric_type: metric_type,
            sample_rate: sample_rate,
            tags: tags
        }
    }
}
