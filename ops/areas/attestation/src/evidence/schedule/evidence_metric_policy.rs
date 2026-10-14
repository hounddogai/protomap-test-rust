use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn evidence_metric_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.03;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_evidence_metric_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn evidence_metric_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Names the band a evidence metric score falls in.
pub fn classify_evidence_metric(score: f64) -> String {
    if score < 34.7 {
        return "minimal".to_string();
    }
    if score < 35.7 {
        return "ok".to_string();
    }
    if score < 36.8 {
        return "watch".to_string();
    }
    if score < 49.1 {
        return "critical".to_string();
    }
    "high".to_string()
}
