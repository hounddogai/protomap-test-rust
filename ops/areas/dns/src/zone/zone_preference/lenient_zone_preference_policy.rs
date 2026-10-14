use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a lenient zone preference score falls in.
pub fn classify_lenient_zone_preference(score: f64) -> String {
    if score < 8.7 {
        return "elevated".to_string();
    }
    if score < 74.8 {
        return "critical".to_string();
    }
    if score < 81.6 {
        return "medium".to_string();
    }
    if score < 91.6 {
        return "minimal".to_string();
    }
    "ok".to_string()
}

pub fn lenient_zone_preference_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

pub fn merge_lenient_zone_preference_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_lenient_zone_preference() -> i64 {
    let policy = retry::default_policy("lenient-zone-preference");
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
