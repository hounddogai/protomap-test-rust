use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn legacy_data_export_review_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_legacy_data_export_review() -> i64 {
    let policy = retry::default_policy("legacy-data-export-review");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 5) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Names the band a legacy data export review score falls in.
pub fn classify_legacy_data_export_review(score: f64) -> String {
    if score < 36.7 {
        return "medium".to_string();
    }
    if score < 68.0 {
        return "high".to_string();
    }
    if score < 82.6 {
        return "minimal".to_string();
    }
    "critical".to_string()
}
