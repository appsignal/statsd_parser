use std::collections::HashMap;
use std::fmt;

mod parser;

pub use parser::ParseError;

#[derive(Debug,PartialEq)]
pub enum MetricType {
    Gauge,
    Counter,
    Timing,
    Histogram,
    Meter
}

impl fmt::Display for MetricType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MetricType::Gauge => write!(f, "gauge"),
            MetricType::Counter => write!(f, "counter"),
            MetricType::Timing => write!(f, "timing"),
            MetricType::Histogram => write!(f, "histogram"),
            MetricType::Meter => write!(f, "meter"),
        }
    }
}

#[derive(Debug,PartialEq)]
pub struct Metric {
    pub name: String,
    pub value: f64,
    pub sample_rate: Option<f64>,
    pub metric_type: MetricType,
    pub tags: Option<HashMap<String, String>>
}

/// Parse a statsd string and return a metric or error message
pub fn parse<S: Into<String>>(input: S) -> Result<Metric, ParseError> {
    parser::Parser::new(input.into()).parse()
}

#[cfg(test)]
mod tests {
    use super::{MetricType, Metric};
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_metric_type_display() {
        assert_eq!("gauge", format!("{}", MetricType::Gauge));
        assert_eq!("counter", format!("{}", MetricType::Counter));
        assert_eq!("timing", format!("{}", MetricType::Timing));
        assert_eq!("histogram", format!("{}", MetricType::Histogram));
        assert_eq!("meter", format!("{}", MetricType::Meter));
    }

    #[test]
    fn test_statsd_counter() {
        let expected = Metric {
            name: "gorets".to_string(),
            value: 1.0,
            metric_type: MetricType::Counter,
            sample_rate: None,
            tags: None
        };

        assert_eq!(parse("gorets:1|c"), Ok(expected));
    }

    #[test]
    fn test_statsd_gauge() {
        let expected = Metric {
            name: "gorets".to_string(),
            value: 1.0,
            metric_type: MetricType::Gauge,
            sample_rate: None,
            tags: None
        };

        assert_eq!(parse("gorets:1|g"), Ok(expected));
    }

    #[test]
    fn test_statsd_time() {
        let expected = Metric {
            name: "gorets".to_string(),
            value: 233.0,
            metric_type: MetricType::Timing,
            sample_rate: None,
            tags: None
        };

        assert_eq!(parse("gorets:233|ms"), Ok(expected));
    }

    #[test]
    fn test_statsd_histogram() {
        let expected = Metric {
            name: "gorets".to_string(),
            value: 233.0,
            metric_type: MetricType::Histogram,
            sample_rate: None,
            tags: None
        };

        assert_eq!(parse("gorets:233|h"), Ok(expected));
    }

    #[test]
    fn test_statsd_meter() {
        let expected = Metric {
            name: "gorets".to_string(),
            value: 233.0,
            metric_type: MetricType::Meter,
            sample_rate: None,
            tags: None
        };

        assert_eq!(parse("gorets:233|m"), Ok(expected));
    }

    #[test]
    fn test_statsd_counter_with_sample_rate() {
        let expected = Metric {
            name: "gorets".to_string(),
            value: 1.0,
            metric_type: MetricType::Counter,
            sample_rate: Some(0.5),
            tags: None
        };

        assert_eq!(parse("gorets:1|c|@0.5"), Ok(expected));
    }

    #[test]
    fn test_statsd_counter_with_tags() {
        let mut tags = HashMap::new();
        tags.insert("foo".to_string(), "bar".to_string());

        let expected = Metric {
            name: "gorets".to_string(),
            value: 1.0,
            metric_type: MetricType::Counter,
            sample_rate: None,
            tags: Some(tags)
        };

        assert_eq!(parse("gorets:1|c|#foo:bar"), Ok(expected));
    }

    #[test]
    fn test_statsd_counter_with_sample_rate_and_tags() {
        let mut tags = HashMap::new();
        tags.insert("foo".to_string(), "bar".to_string());
        tags.insert("moo".to_string(), "maa".to_string());

        let expected = Metric {
            name: "gorets".to_string(),
            value: 1.0,
            metric_type: MetricType::Counter,
            sample_rate: Some(0.9),
            tags: Some(tags)
        };

        assert_eq!(parse("gorets:1|c|@0.9|#foo:bar,moo:maa"), Ok(expected));
    }

    #[test]
    fn test_statsd_utf8_boundary() {
        let expected = Metric {
            name: "goretsβ".to_string(),
            value: 1.0,
            metric_type: MetricType::Counter,
            sample_rate: None,
            tags: None
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
