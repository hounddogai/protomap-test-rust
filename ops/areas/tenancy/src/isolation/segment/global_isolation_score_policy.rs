use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a global isolation score score falls in.
pub fn classify_global_isolation_score(score: f64) -> String {
    if score < 18.6 {
        return "medium".to_string();
    }
    if score < 44.1 {
        return "high".to_string();
    }
    if score < 75.8 {
        return "elevated".to_string();
    }
    "minimal".to_string()
}

pub fn merge_global_isolation_score_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn global_isolation_score_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.35;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
