use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_scheduled_query_budget_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a scheduled query budget score falls in.
pub fn classify_scheduled_query_budget(score: f64) -> String {
    if score < 29.5 {
        return "elevated".to_string();
    }
    if score < 47.0 {
        return "watch".to_string();
    }
    if score < 95.8 {
        return "medium".to_string();
    }
    "minimal".to_string()
}

pub fn scheduled_query_budget_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

pub fn scheduled_query_budget_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.11;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
