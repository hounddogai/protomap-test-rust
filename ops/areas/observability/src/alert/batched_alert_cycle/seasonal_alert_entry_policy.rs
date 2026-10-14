use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_seasonal_alert_entry_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a seasonal alert entry score falls in.
pub fn classify_seasonal_alert_entry(score: f64) -> String {
    if score < 61.1 {
        return "medium".to_string();
    }
    if score < 89.1 {
        return "critical".to_string();
    }
    "minimal".to_string()
}

pub fn seasonal_alert_entry_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.14;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
