use std::collections::BTreeMap;

mod parser;

pub use parser::ParseError;

#[derive(Debug,PartialEq)]
pub struct Message {
    pub name: String,
    pub tags: Option<BTreeMap<String, String>>,
    pub metric: Metric
}

#[derive(Debug,PartialEq)]
pub enum Metric {
    Gauge(Gauge),
    Counter(Counter),
    Timing(Timing),
    Histogram(Histogram),
    Meter(Meter),
    Distribution(Distribution),
    Set(Set),
    ServiceCheck(ServiceCheck)
}

#[derive(Debug,PartialEq)]
pub enum Status {
    OK,
    WARNING,
    CRITICAL,
    UNKNOWN
}

#[derive(Debug,PartialEq)]
pub struct Gauge {
    pub value: f64,
    pub sample_rate: Option<f64>,
}

#[derive(Debug,PartialEq)]
pub struct Counter {
    pub value: f64,
    pub sample_rate: Option<f64>,
}

#[derive(Debug,PartialEq)]
pub struct Timing {
    pub value: f64,
    pub sample_rate: Option<f64>,
}

#[derive(Debug,PartialEq)]
pub struct Histogram {
    pub value: f64,
    pub sample_rate: Option<f64>,
}

#[derive(Debug,PartialEq)]
pub struct Meter {
    pub value: f64,
    pub sample_rate: Option<f64>,
}

#[derive(Debug,PartialEq)]
pub struct Distribution {
    pub value: f64,
    pub sample_rate: Option<f64>,
}

#[derive(Debug,PartialEq)]
pub struct Set {
    pub value: f64,
    pub sample_rate: Option<f64>,
}

#[derive(Debug,PartialEq)]
pub struct ServiceCheck {
    pub status: Status,
    pub timestamp: Option<f64>,
    pub hostname: Option<String>,
    pub message: Option<String>,
}

/// Parse a statsd string and return a metric or error message
pub fn parse<S: Into<String>>(input: S) -> Result<Message, ParseError> {
    let string = input.into();

    if string.starts_with("_sc") {
        parser::service_check_parser::parse(string)
    } else {
        parser::metric_parser::parse(string)
    }
}

#[cfg(test)]
mod tests {
    use {Message, Metric};
    use std::collections::BTreeMap;

    use super::*;

    #[test]
    fn test_statsd_counter() {
        let expected = Message {
            name: "gorets".to_string(),
            tags: None,
            metric: Metric::Counter(Counter {
                value: 1.0,
                sample_rate: None,
            })
        };

        assert_eq!(parse("gorets:1|c"), Ok(expected));
    }

    #[test]
    fn test_statsd_counter_newline() {
        let expected = Message {
            name: "gorets".to_string(),
            tags: None,
            metric: Metric::Counter(Counter {
                value: 1.0,
                sample_rate: None,
            })
        };

        assert_eq!(parse("gorets:1|c\n"), Ok(expected));
    }

    #[test]
    fn test_statsd_gauge() {
        let expected = Message {
            name: "gorets".to_string(),
            tags: None,
            metric: Metric::Gauge(Gauge {
                value: 1.0,
                sample_rate: None,
            })
        };

        assert_eq!(parse("gorets:1|g"), Ok(expected));
    }

    #[test]
    fn test_statsd_time() {
        let expected = Message {
            name: "gorets".to_string(),
            tags: None,
            metric: Metric::Timing(Timing {
                value: 233.0,
                sample_rate: None,
            })
        };

        assert_eq!(parse("gorets:233|ms"), Ok(expected));
    }

    #[test]
    fn test_statsd_histogram() {
        let expected = Message {
            name: "gorets".to_string(),
            tags: None,
            metric: Metric::Histogram(Histogram {
                value: 233.0,
                sample_rate: None,
            })
        };

        assert_eq!(parse("gorets:233|h"), Ok(expected));
    }

    #[test]
    fn test_statsd_distribution() {
        let expected = Message {
            name: "gorets".to_string(),
            tags: None,
            metric: Metric::Distribution(Distribution {
                value: 233.0,
                sample_rate: None,
            })
        };

        assert_eq!(parse("gorets:233|d"), Ok(expected));
    }

    #[test]
    fn test_statsd_set() {
        let expected = Message {
            name: "gorets".to_string(),
            tags: None,
            metric: Metric::Set(Set {
                value: 233.0,
                sample_rate: None,
            })
        };

        assert_eq!(parse("gorets:233|s"), Ok(expected));
    }

    #[test]
    fn test_statsd_meter() {
        let expected = Message {
            name: "gorets".to_string(),
            tags: None,
            metric: Metric::Meter(Meter {
                value: 233.0,
                sample_rate: None,
            })
        };

        assert_eq!(parse("gorets:233|m"), Ok(expected));
    }

    #[test]
    fn test_statsd_counter_with_sample_rate() {
        let expected = Message {
            name: "gorets".to_string(),
            tags: None,
            metric: Metric::Counter(Counter {
                value: 1.0,
                sample_rate: Some(0.5),
            })
        };

        assert_eq!(parse("gorets:1|c|@0.5"), Ok(expected));
    }

    #[test]
    fn test_statsd_counter_with_key_value_tags() {
        let mut tags = BTreeMap::new();
        tags.insert("foo".to_string(), "bar".to_string());

        let expected = Message {
            name: "gorets".to_string(),
            tags: Some(tags),
            metric: Metric::Counter(Counter {
                value: 1.0,
                sample_rate: None,
            })
        };

        assert_eq!(parse("gorets:1|c|#foo:bar"), Ok(expected));
    }

    #[test]
    fn test_statsd_counter_with_key_tags() {
        let mut tags = BTreeMap::new();
        tags.insert("foo".to_string(), "".to_string());
        tags.insert("moo".to_string(), "".to_string());

        let expected = Message {
            name: "gorets".to_string(),
            tags: Some(tags),
            metric: Metric::Counter(Counter {
                value: 1.0,
                sample_rate: None,
            })
        };

        assert_eq!(parse("gorets:1|c|#foo,moo"), Ok(expected));
    }

    #[test]
    fn test_statsd_counter_with_sample_rate_and_tags() {
        let mut tags = BTreeMap::new();
        tags.insert("foo".to_string(), "bar".to_string());
        tags.insert("moo".to_string(), "maa".to_string());

        let expected = Message {
            name: "gorets".to_string(),
            tags: Some(tags),
            metric: Metric::Counter(Counter {
                value: 1.0,
                sample_rate: Some(0.9),
            })
        };

        assert_eq!(parse("gorets:1|c|@0.9|#foo:bar,moo:maa"), Ok(expected));
    }

    #[test]
    fn test_statsd_utf8_boundary() {
        let expected = Message {
            name: "goretsβ".to_string(),
            tags: None,
            metric: Metric::Counter(Counter {
                value: 1.0,
                sample_rate: None,
            })
        };

        assert_eq!(parse("goretsβ:1|c"), Ok(expected));
    }

    #[test]
    fn test_statsd_empty() {
        assert_eq!(parse(""), Err(ParseError::EmptyInput));
    }

    #[test]
    fn test_statsd_no_name() {
        assert_eq!(parse(":1|c"), Err(ParseError::NoName));
    }

    #[test]
    fn test_statsd_value_not_float() {
        assert_eq!(parse("gorets:aaa|h"), Err(ParseError::ValueNotFloat));
    }

    #[test]
    fn test_statsd_sample_rate_not_float() {
        assert_eq!(parse("gorets:1|c|@aaa"), Err(ParseError::SampleRateNotFloat));
    }

    #[test]
    fn test_statsd_metric_type_unknown() {
        assert_eq!(parse("gorets:1|wrong"), Err(ParseError::UnknownMetricType));
    }
}
