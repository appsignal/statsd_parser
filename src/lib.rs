use std::collections::HashMap;

mod parser;

#[derive(Debug,PartialEq)]
pub enum MetricType {
    Gauge,
    Counter,
    Timing,
    Histogram,
    Meter,
    Unknown(String)
}

#[derive(Debug,PartialEq)]
pub struct Metric {
    name: String,
    value: f64,
    sample_rate: f64,
    metric_type: MetricType,
    tags: HashMap<String, String>
}

pub fn parse<S: Into<String>>(input: S) -> Metric {
    parser::Parser::new(input.into()).parse()
}

#[cfg(test)]
mod tests {
    use super::{MetricType, Metric};
    use std::collections::HashMap;

    use super::parse;

    #[test]
    fn test_statsd_counter() {
        let expected = Metric {
            name: "gorets".to_string(),
            value: 1.0,
            metric_type: MetricType::Counter,
            sample_rate: 0.0,
            tags: HashMap::new()
        };

        assert_eq!(parse("gorets:1|c"), expected);
    }

    #[test]
    fn test_statsd_gauge() {
        let expected = Metric {
            name: "gorets".to_string(),
            value: 1.0,
            metric_type: MetricType::Gauge,
            sample_rate: 0.0,
            tags: HashMap::new()
        };

        assert_eq!(parse("gorets:1|g"), expected);
    }

    #[test]
    fn test_statsd_time() {
        let expected = Metric {
            name: "gorets".to_string(),
            value: 233.0,
            metric_type: MetricType::Timing,
            sample_rate: 0.0,
            tags: HashMap::new()
        };

        assert_eq!(parse("gorets:233|ms"), expected);
    }

    #[test]
    fn test_statsd_histogram() {
        let expected = Metric {
            name: "gorets".to_string(),
            value: 233.0,
            metric_type: MetricType::Histogram,
            sample_rate: 0.0,
            tags: HashMap::new()
        };

        assert_eq!(parse("gorets:233|h"), expected);
    }

    #[test]
    fn test_statsd_meter() {
        let expected = Metric {
            name: "gorets".to_string(),
            value: 233.0,
            metric_type: MetricType::Meter,
            sample_rate: 0.0,
            tags: HashMap::new()
        };

        assert_eq!(parse("gorets:233|m"), expected);
    }

    #[test]
    fn test_unknown_metric_type() {
        let expected = Metric {
            name: "gorets".to_string(),
            value: 1.0,
            metric_type: MetricType::Unknown("wrong".to_string()),
            sample_rate: 0.0,
            tags: HashMap::new()
        };

        assert_eq!(parse("gorets:1|wrong"), expected);
    }

    #[test]
    fn test_statsd_counter_with_sample_rate() {
        let expected = Metric {
            name: "gorets".to_string(),
            value: 1.0,
            metric_type: MetricType::Counter,
            sample_rate: 0.5,
            tags: HashMap::new()
        };

        assert_eq!(parse("gorets:1|c|@0.5"), expected);
    }

    #[test]
    fn test_statsd_counter_with_tags() {
        let mut tags = HashMap::new();
        tags.insert("foo".to_string(), "bar".to_string());

        let expected = Metric {
            name: "gorets".to_string(),
            value: 1.0,
            metric_type: MetricType::Counter,
            sample_rate: 0.0,
            tags: tags
        };

        assert_eq!(parse("gorets:1|c|#foo:bar"), expected);
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
            sample_rate: 0.9,
            tags: tags
        };

        assert_eq!(parse("gorets:1|c|@0.9|#foo:bar,moo:maa"), expected);
    }
}
