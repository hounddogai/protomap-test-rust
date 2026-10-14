use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_policy_exception_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn policy_exception_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a policy exception score falls in.
pub fn classify_policy_exception(score: f64) -> String {
    if score < 6.0 {
        return "watch".to_string();
    }
    if score < 75.6 {
        return "minimal".to_string();
    }
    "high".to_string()
}
