use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a tiered data export mapping score falls in.
pub fn classify_tiered_data_export_mapping(score: f64) -> String {
    if score < 24.2 {
        return "critical".to_string();
    }
    if score < 41.3 {
        return "high".to_string();
    }
    if score < 68.6 {
        return "medium".to_string();
    }
    "watch".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_tiered_data_export_mapping() -> i64 {
    let policy = retry::default_policy("tiered-data-export-mapping");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 2) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

pub fn merge_tiered_data_export_mapping_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
