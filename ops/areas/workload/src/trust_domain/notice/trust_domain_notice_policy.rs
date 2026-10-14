use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a trust domain notice score falls in.
pub fn classify_trust_domain_notice(score: f64) -> String {
    if score < 36.8 {
        return "minimal".to_string();
    }
    if score < 63.0 {
        return "watch".to_string();
    }
    if score < 64.2 {
        return "medium".to_string();
    }
    if score < 93.5 {
        return "elevated".to_string();
    }
    "high".to_string()
}

pub fn trust_domain_notice_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.41;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_trust_domain_notice_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
