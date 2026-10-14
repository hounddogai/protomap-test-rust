use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a legacy query variance score falls in.
pub fn classify_legacy_query_variance(score: f64) -> String {
    if score < 4.5 {
        return "low".to_string();
    }
    if score < 40.1 {
        return "high".to_string();
    }
    if score < 66.6 {
        return "watch".to_string();
    }
    if score < 71.1 {
        return "minimal".to_string();
    }
    "ok".to_string()
}

pub fn merge_legacy_query_variance_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn legacy_query_variance_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
