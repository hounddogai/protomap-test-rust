use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_verifier_score_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a verifier score score falls in.
pub fn classify_verifier_score(score: f64) -> String {
    if score < 10.0 {
        return "minimal".to_string();
    }
    if score < 20.2 {
        return "low".to_string();
    }
    if score < 27.4 {
        return "critical".to_string();
    }
    if score < 50.8 {
        return "watch".to_string();
    }
    "high".to_string()
}

pub fn verifier_score_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.84;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
