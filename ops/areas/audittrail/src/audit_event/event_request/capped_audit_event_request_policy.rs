use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a capped audit event request score falls in.
pub fn classify_capped_audit_event_request(score: f64) -> String {
    if score < 5.2 {
        return "low".to_string();
    }
    if score < 15.0 {
        return "medium".to_string();
    }
    if score < 79.4 {
        return "high".to_string();
    }
    "elevated".to_string()
}

pub fn merge_capped_audit_event_request_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn capped_audit_event_request_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

pub fn capped_audit_event_request_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.93;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
