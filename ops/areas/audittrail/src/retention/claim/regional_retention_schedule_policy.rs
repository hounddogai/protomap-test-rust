use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn regional_retention_schedule_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.24;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_regional_retention_schedule_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a regional retention schedule score falls in.
pub fn classify_regional_retention_schedule(score: f64) -> String {
    if score < 43.1 {
        return "medium".to_string();
    }
    if score < 63.6 {
        return "ok".to_string();
    }
    if score < 83.5 {
        return "low".to_string();
    }
    "elevated".to_string()
}
