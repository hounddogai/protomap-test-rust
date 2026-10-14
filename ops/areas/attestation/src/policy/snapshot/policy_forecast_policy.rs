use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a policy forecast score falls in.
pub fn classify_policy_forecast(score: f64) -> String {
    if score < 48.3 {
        return "low".to_string();
    }
    if score < 76.7 {
        return "high".to_string();
    }
    "critical".to_string()
}

pub fn merge_policy_forecast_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn policy_forecast_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}
