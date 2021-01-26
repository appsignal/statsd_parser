use {Message, Metric, Gauge, Counter, Timing, Histogram, Meter, Distribution, Set};
use super::{Parser, ParseError};

pub trait MetricParser {
    fn parse(self) -> Result<Message, ParseError>;
}

impl MetricParser for Parser {
    fn parse(mut self) -> Result<Message, ParseError> {
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

        let metric = match metric_type.as_ref() {
            "ms" => {
                Metric::Timing(Timing {
                    value: value,
                    sample_rate: sample_rate,
                })
            },
            "c" => {
                Metric::Counter(Counter {
                    value: value,
                    sample_rate: sample_rate,
                })
            },
            "g" => {
                Metric::Gauge(Gauge {
                    value: value,
                    sample_rate: sample_rate,
                })
            },
            "m" => {
                Metric::Meter(Meter {
                    value: value,
                    sample_rate: sample_rate,
                })
            },
            "h" => {
                Metric::Histogram(Histogram {
                    value: value,
                    sample_rate: sample_rate,
                })
            },
            "d" => {
                Metric::Distribution(Distribution {
                    value: value,
                    sample_rate: sample_rate,
                })
            },
            "s" => {
                Metric::Set(Set {
                    value: value,
                    sample_rate: sample_rate,                        
                })
            }
            _ => return Err(ParseError::UnknownMetricType)
        };

        Ok(Message {
            name: name,
            tags: tags,
            metric: metric
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
    use {Message, Metric, Timing};

    #[test]
    fn test_parse_with_tags() {
        let result = parse("service.duration:101|ms|@0.9|#hostname:frontend1,namespace:web".to_string());

        let mut tags = BTreeMap::new();
        tags.insert("hostname".to_string(), "frontend1".to_string());
        tags.insert("namespace".to_string(), "web".to_string());

        let expected = Message {
            name: "service.duration".to_string(),
            tags: Some(tags),
            metric: Metric::Timing(Timing {
                value: 101.0,
                sample_rate: Some(0.9),
            })
        };

        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_parse_without_tags() {
        let result = parse("service.duration:101|ms|@0.9|".to_string());

        let expected = Message {
            name: "service.duration".to_string(),
            tags: None,
            metric: Metric::Timing(Timing {
                value: 101.0,
                sample_rate: Some(0.9),
            })
        };

        assert_eq!(result, Ok(expected));
    }

    #[test]
    fn test_parse_invalid() {
        let result = parse("service.duration:101|aaa|@0.9|".to_string());
        assert!(result.is_err());
    }
}
