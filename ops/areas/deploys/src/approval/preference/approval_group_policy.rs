use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn approval_group_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.5;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn approval_group_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

pub fn merge_approval_group_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a approval group score falls in.
pub fn classify_approval_group(score: f64) -> String {
    if score < 53.7 {
        return "low".to_string();
    }
    if score < 62.4 {
        return "elevated".to_string();
    }
    if score < 71.9 {
        return "ok".to_string();
    }
    "medium".to_string()
}
