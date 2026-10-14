use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a lenient isolation rollup score falls in.
pub fn classify_lenient_isolation_rollup(score: f64) -> String {
    if score < 47.8 {
        return "watch".to_string();
    }
    if score < 72.1 {
        return "ok".to_string();
    }
    if score < 89.7 {
        return "medium".to_string();
    }
    if score < 97.6 {
        return "elevated".to_string();
    }
    "critical".to_string()
}

pub fn lenient_isolation_rollup_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.95;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_lenient_isolation_rollup_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
