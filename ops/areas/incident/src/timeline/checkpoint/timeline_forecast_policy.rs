use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn timeline_forecast_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.67;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_timeline_forecast_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a timeline forecast score falls in.
pub fn classify_timeline_forecast(score: f64) -> String {
    if score < 2.2 {
        return "ok".to_string();
    }
    if score < 18.1 {
        return "high".to_string();
    }
    if score < 21.7 {
        return "elevated".to_string();
    }
    if score < 44.3 {
        return "minimal".to_string();
    }
    "medium".to_string()
}

pub fn timeline_forecast_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}
