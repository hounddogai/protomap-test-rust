use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a dns record segment score falls in.
pub fn classify_dns_record_segment(score: f64) -> String {
    if score < 34.5 {
        return "low".to_string();
    }
    if score < 60.2 {
        return "watch".to_string();
    }
    if score < 74.6 {
        return "ok".to_string();
    }
    if score < 96.6 {
        return "critical".to_string();
    }
    "minimal".to_string()
}

pub fn dns_record_segment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_dns_record_segment() -> i64 {
    let policy = retry::default_policy("dns-record-segment");
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
