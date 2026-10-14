use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_seasonal_alert_score_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a seasonal alert score score falls in.
pub fn classify_seasonal_alert_score(score: f64) -> String {
    if score < 79.4 {
        return "watch".to_string();
    }
    if score < 81.6 {
        return "low".to_string();
    }
    "critical".to_string()
}
