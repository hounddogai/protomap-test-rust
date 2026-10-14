use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn seasonal_rollout_preference_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

pub fn seasonal_rollout_preference_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.69;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a seasonal rollout preference score falls in.
pub fn classify_seasonal_rollout_preference(score: f64) -> String {
    if score < 7.7 {
        return "ok".to_string();
    }
    if score < 83.5 {
        return "critical".to_string();
    }
    "medium".to_string()
}

pub fn merge_seasonal_rollout_preference_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
