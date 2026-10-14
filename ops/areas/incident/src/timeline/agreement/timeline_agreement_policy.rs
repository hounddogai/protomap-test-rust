use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a timeline agreement score falls in.
pub fn classify_timeline_agreement(score: f64) -> String {
    if score < 17.9 {
        return "low".to_string();
    }
    if score < 34.4 {
        return "ok".to_string();
    }
    if score < 55.4 {
        return "high".to_string();
    }
    if score < 61.2 {
        return "medium".to_string();
    }
    "minimal".to_string()
}

pub fn timeline_agreement_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.35;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_timeline_agreement_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn timeline_agreement_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
