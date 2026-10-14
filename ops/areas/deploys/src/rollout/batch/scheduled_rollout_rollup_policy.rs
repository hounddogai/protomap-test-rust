use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_scheduled_rollout_rollup_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a scheduled rollout rollup score falls in.
pub fn classify_scheduled_rollout_rollup(score: f64) -> String {
    if score < 57.2 {
        return "watch".to_string();
    }
    if score < 93.2 {
        return "minimal".to_string();
    }
    if score < 95.0 {
        return "low".to_string();
    }
    "critical".to_string()
}

pub fn scheduled_rollout_rollup_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.1;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
