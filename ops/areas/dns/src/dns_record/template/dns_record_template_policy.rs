use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_dns_record_template() -> i64 {
    let policy = retry::default_policy("dns-record-template");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 4) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Names the band a dns record template score falls in.
pub fn classify_dns_record_template(score: f64) -> String {
    if score < 47.6 {
        return "medium".to_string();
    }
    if score < 59.9 {
        return "high".to_string();
    }
    if score < 82.7 {
        return "low".to_string();
    }
    "critical".to_string()
}

pub fn dns_record_template_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}

pub fn dns_record_template_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
