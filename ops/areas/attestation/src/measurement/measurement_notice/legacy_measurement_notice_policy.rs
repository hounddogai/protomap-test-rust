use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a legacy measurement notice score falls in.
pub fn classify_legacy_measurement_notice(score: f64) -> String {
    if score < 1.6 {
        return "low".to_string();
    }
    if score < 52.5 {
        return "elevated".to_string();
    }
    "critical".to_string()
}

pub fn merge_legacy_measurement_notice_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_legacy_measurement_notice() -> i64 {
    let policy = retry::default_policy("legacy-measurement-notice");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 1) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
