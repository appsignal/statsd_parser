# (Dog)statsD Parser

[![Build Status](https://travis-ci.org/appsignal/statsd_parser.svg?branch=master)](https://travis-ci.org/appsignal/statsd_parser)
[![Crate](http://meritbadge.herokuapp.com/statsd_parser)](https://crates.io/crates/statsd_parser)

Parses (part of) the [(Dog)StatsD protocol](https://docs.datadoghq.com/guides/dogstatsd/) and returns a struct with the values:


```rust
#[test]
fn test_statsd_counter_with_sample_rate_and_tags() {
    let mut tags = HashMap::new();
    tags.insert("foo".to_string(), "bar".to_string());
    tags.insert("moo".to_string(), "maa".to_string());

    let expected = ParseResult {
        name: "gorets".to_string(),
        value: 1.0,
        metric_type: MetricType::Counter,
        sample_rate: 0.9,
        tags: tags
    };

    assert_eq!(parse("gorets:1|c|@0.9|#foo:bar,moo:maa"), expected);
}
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Contributions are very welcome. Please make sure that you add a test for any use case you want to add.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
