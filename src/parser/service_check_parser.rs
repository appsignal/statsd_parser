use {Message, Metric, ServiceCheck, Status};
use super::{Parser, ParseError};

pub trait ServiceStatusParser {
    fn parse(self) -> Result<Message, ParseError>;
}

impl ServiceStatusParser for Parser {
    fn parse(mut self) -> Result<Message, ParseError> {
        if self.chars.is_empty() {
            return Err(ParseError::EmptyInput)
        }

        // Start with the service check tag
        self.take_until(vec!['|']);

        // Get the name
        let name = self.take_until(vec!['|']);
        if name.is_empty() {
            return Err(ParseError::NoName)
        }

        // Get the status
        let status = match self.take_until(vec!['|']).as_ref() {
            "0" => Status::OK,
            "1" => Status::WARNING,
            "2" => Status::CRITICAL,
            _ => Status::UNKNOWN
        };

        // Peek the string to see if we need to parse a timestamp
        let timestamp = if Some('d') == self.peek() {
            self.skip();
            self.skip();
            match self.take_float_until(vec!['|']) {
                Ok(v) => Some(v),
                Err(_) => return Err(ParseError::ValueNotFloat)
            }
        } else {
            None
        };

        // Peek the string to see if we need to parse a hostname
        let hostname = if Some('h') == self.peek() {
            self.skip();
            self.skip();
            Some(self.take_until(vec!['|']))
        } else {
            None
        };

        // Peek the string to see if we need to parse tags
        let tags = if Some('#') == self.peek() {
            Some(self.parse_tags())
        } else {
            None
        };

        // Peek the string to see if we need to parse a message
        let message = if Some('m') == self.peek() {
            self.skip();
            self.skip();
            Some(self.take_until(vec!['|']))
        } else {
            None
        };

        let service_check = ServiceCheck {
            status: status,
            timestamp: timestamp,
            hostname: hostname,
            message: message
        };

        Ok(Message {
            name: name,
            tags: tags,
            metric: Metric::ServiceCheck(service_check)
        })
    }
}

pub fn parse(input: String) -> Result<Message, ParseError> {
    Parser::new(input).parse()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::parse;
    use {Message, Metric, ServiceCheck, Status};

    #[test]
    fn test_parse_with_tags() {
        let result = parse("_sc|Redis connection|2|d:10101|h:frontend1|#redis_instance:10.0.0.16:6379|m:Redis connection timed out after 10s".to_string());

        let mut tags = BTreeMap::new();
        tags.insert("redis_instance".to_string(), "10.0.0.16:6379".to_string());

        let expected = Message {
            name: "Redis connection".to_string(),
            tags: Some(tags),
            metric: Metric::ServiceCheck(ServiceCheck {
                status: Status::CRITICAL,
                timestamp: Some(10101f64),
                hostname: Some("frontend1".to_string()),
                message: Some("Redis connection timed out after 10s".to_string()),
            })
        };

        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_parse_without_tags() {
        let result = parse("_sc|Redis connection|0|d:10101|h:frontend1|m:Redis connection timed out after 10s".to_string());

        let expected = Message {
            name: "Redis connection".to_string(),
            tags: None,
            metric: Metric::ServiceCheck(ServiceCheck {
                status: Status::OK,
                timestamp: Some(10101f64),
                hostname: Some("frontend1".to_string()),
                message: Some("Redis connection timed out after 10s".to_string()),
            })
        };

        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_parse_without_duration() {
        let result = parse("_sc|Redis connection|1|h:frontend1|m:Redis connection timed out after 10s".to_string());

        let expected = Message {
            name: "Redis connection".to_string(),
            tags: None,
            metric: Metric::ServiceCheck(ServiceCheck {
                status: Status::WARNING,
                timestamp: None,
                hostname: Some("frontend1".to_string()),
                message: Some("Redis connection timed out after 10s".to_string()),
            })
        };

        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_parse_minimum_required() {
        let result = parse("_sc|Redis connection".to_string());

        let expected = Message {
            name: "Redis connection".to_string(),
            tags: None,
            metric:  Metric::ServiceCheck(ServiceCheck {
                status: Status::UNKNOWN,
                timestamp: None,
                hostname: None,
                message: None,
            })
        };

        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_parse_invalid() {
        let result = parse("Redis connection".to_string());
        println!("{:?}", result);
        assert!(result.is_err());
    }
}
