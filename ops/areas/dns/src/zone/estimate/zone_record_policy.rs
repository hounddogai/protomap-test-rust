use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_zone_record() -> i64 {
    let policy = retry::default_policy("zone-record");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 3) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Names the band a zone record score falls in.
pub fn classify_zone_record(score: f64) -> String {
    if score < 10.0 {
        return "high".to_string();
    }
    if score < 12.3 {
        return "watch".to_string();
    }
    if score < 37.8 {
        return "low".to_string();
    }
    if score < 39.7 {
        return "medium".to_string();
    }
    "elevated".to_string()
}

pub fn zone_record_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}
