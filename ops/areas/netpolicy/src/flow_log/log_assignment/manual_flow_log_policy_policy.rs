use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_manual_flow_log_policy_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn manual_flow_log_policy_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.95;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a manual flow log policy score falls in.
pub fn classify_manual_flow_log_policy(score: f64) -> String {
    if score < 51.5 {
        return "medium".to_string();
    }
    if score < 56.6 {
        return "critical".to_string();
    }
    if score < 65.6 {
        return "watch".to_string();
    }
    if score < 93.9 {
        return "elevated".to_string();
    }
    "ok".to_string()
}
