use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a alert preference score falls in.
pub fn classify_alert_preference(score: f64) -> String {
    if score < 29.7 {
        return "critical".to_string();
    }
    if score < 56.3 {
        return "minimal".to_string();
    }
    if score < 76.6 {
        return "medium".to_string();
    }
    if score < 90.9 {
        return "watch".to_string();
    }
    "high".to_string()
}

pub fn merge_alert_preference_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
