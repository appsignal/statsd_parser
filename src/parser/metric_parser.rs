use {Metric, Gauge, Timing, Counter, Meter, Histogram};
use super::{Parser, ParseError};

pub trait MetricParser {
    fn parse(self) -> Result<Metric, ParseError>;
}

impl MetricParser for Parser {
    fn parse(mut self) -> Result<Metric, ParseError> {
        if self.chars.is_empty() {
            return Err(ParseError::EmptyInput)
        }

        // Start with the name
        let name = self.take_until(vec![':']);

        if name.is_empty() {
            return Err(ParseError::NoName)
        }

        // The value should be everything until the first pipe (`|`)
        let value = match self.take_float_until(vec!['|']) {
            Ok(v) => v,
            Err(_) => return Err(ParseError::ValueNotFloat)
        };

        // The metric type should be everything until the next pipe, or the end
        let metric_type = self.take_until(vec!['|']);

        // The next part can either be the sample rate or tags,
        // peek the value and match on `@` to get the sample rate
        let sample_rate = match self.peek() {
            Some('@') => {
                self.skip(); // Skip the `@`
                match self.take_float_until(vec!['|']) {
                    Ok(v) => Some(v),
                    Err(_) => return Err(ParseError::SampleRateNotFloat)
                }
            }
            _ => None
        };

        // Peek the remaining string, if it starts with a pound (`#`)
        // try and match tags
        let tags = if Some('#') == self.peek() {
            Some(self.parse_tags())
        } else {
            None
        };

        match metric_type.as_ref() {
            "ms" => {
                Ok(Metric::Timing(Timing {
                    name: name,
                    value: value,
                    sample_rate: sample_rate,
                    tags: tags
                }))
            },
            "c" => {
                Ok(Metric::Counter(Counter {
                    name: name,
                    value: value,
                    sample_rate: sample_rate,
                    tags: tags
                }))
            },
            "g" => {
                Ok(Metric::Gauge(Gauge {
                    name: name,
                    value: value,
                    sample_rate: sample_rate,
                    tags: tags
                }))
            },
            "m" => {
                Ok(Metric::Meter(Meter {
                    name: name,
                    value: value,
                    sample_rate: sample_rate,
                    tags: tags
                }))
            },
            "h" => {
                Ok(Metric::Histogram(Histogram {
                    name: name,
                    value: value,
                    sample_rate: sample_rate,
                    tags: tags
                }))
            },
            _ => Err(ParseError::UnknownMetricType)
        }
    }
}

pub fn parse(input: String) -> Result<Metric, ParseError> {
    Parser::new(input).parse()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::parse;
    use {Metric, Timing};

    #[test]
    fn test_parse_with_tags() {
        let result = parse("service.duration:101|ms|@0.9|#hostname:frontend1,namespace:web".to_string());

        let mut tags = BTreeMap::new();
        tags.insert("hostname".to_string(), "frontend1".to_string());
        tags.insert("namespace".to_string(), "web".to_string());

        let expected = Metric::Timing(Timing {
            name: "service.duration".to_string(),
            value: 101.0,
            sample_rate: Some(0.9),
            tags: Some(tags)
        });

        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_parse_without_tags() {
        let result = parse("service.duration:101|ms|@0.9|".to_string());

        let expected = Metric::Timing(Timing {
            name: "service.duration".to_string(),
            value: 101.0,
            sample_rate: Some(0.9),
            tags: None
        });

        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_parse_invalid() {
        let result = parse("service.duration:101|aaa|@0.9|".to_string());
        assert!(result.is_err());
    }
}
