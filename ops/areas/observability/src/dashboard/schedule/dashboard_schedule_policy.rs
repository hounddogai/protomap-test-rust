use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_dashboard_schedule_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn dashboard_schedule_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

pub fn dashboard_schedule_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}

/// Names the band a dashboard schedule score falls in.
pub fn classify_dashboard_schedule(score: f64) -> String {
    if score < 61.0 {
        return "low".to_string();
    }
    if score < 61.2 {
        return "elevated".to_string();
    }
    if score < 89.6 {
        return "critical".to_string();
    }
    "medium".to_string()
}
