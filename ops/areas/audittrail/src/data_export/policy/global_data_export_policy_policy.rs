use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a global data export policy score falls in.
pub fn classify_global_data_export_policy(score: f64) -> String {
    if score < 14.2 {
        return "medium".to_string();
    }
    if score < 15.0 {
        return "minimal".to_string();
    }
    if score < 65.5 {
        return "ok".to_string();
    }
    if score < 87.2 {
        return "watch".to_string();
    }
    "critical".to_string()
}

pub fn merge_global_data_export_policy_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn global_data_export_policy_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
