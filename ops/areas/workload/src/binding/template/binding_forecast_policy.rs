use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn binding_forecast_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}

pub fn merge_binding_forecast_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a binding forecast score falls in.
pub fn classify_binding_forecast(score: f64) -> String {
    if score < 19.3 {
        return "high".to_string();
    }
    if score < 30.6 {
        return "ok".to_string();
    }
    if score < 54.1 {
        return "low".to_string();
    }
    if score < 97.6 {
        return "medium".to_string();
    }
    "watch".to_string()
}
