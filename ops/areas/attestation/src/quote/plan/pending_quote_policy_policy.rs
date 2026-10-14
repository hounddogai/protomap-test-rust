use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn pending_quote_policy_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.36;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_pending_quote_policy_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn pending_quote_policy_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a pending quote policy score falls in.
pub fn classify_pending_quote_policy(score: f64) -> String {
    if score < 17.8 {
        return "watch".to_string();
    }
    if score < 56.7 {
        return "medium".to_string();
    }
    if score < 71.4 {
        return "critical".to_string();
    }
    if score < 71.4 {
        return "high".to_string();
    }
    "low".to_string()
}
