use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn credential_forecast_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}

/// Names the band a credential forecast score falls in.
pub fn classify_credential_forecast(score: f64) -> String {
    if score < 48.0 {
        return "critical".to_string();
    }
    if score < 84.2 {
        return "medium".to_string();
    }
    "watch".to_string()
}

pub fn merge_credential_forecast_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
