use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a fallback quote schedule score falls in.
pub fn classify_fallback_quote_schedule(score: f64) -> String {
    if score < 2.8 {
        return "elevated".to_string();
    }
    if score < 39.2 {
        return "medium".to_string();
    }
    if score < 62.6 {
        return "critical".to_string();
    }
    "minimal".to_string()
}

pub fn fallback_quote_schedule_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.91;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_fallback_quote_schedule_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
