use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_verifier_schedule_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn verifier_schedule_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.66;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a verifier schedule score falls in.
pub fn classify_verifier_schedule(score: f64) -> String {
    if score < 41.3 {
        return "ok".to_string();
    }
    if score < 44.5 {
        return "critical".to_string();
    }
    if score < 44.6 {
        return "low".to_string();
    }
    "high".to_string()
}
