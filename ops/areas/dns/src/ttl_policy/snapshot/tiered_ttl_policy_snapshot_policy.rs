use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a tiered ttl policy snapshot score falls in.
pub fn classify_tiered_ttl_policy_snapshot(score: f64) -> String {
    if score < 16.3 {
        return "minimal".to_string();
    }
    if score < 48.0 {
        return "low".to_string();
    }
    if score < 76.4 {
        return "watch".to_string();
    }
    if score < 85.5 {
        return "medium".to_string();
    }
    "high".to_string()
}

/// Puts a tiered ttl policy snapshot code in its canonical TTPS- form.
pub fn normalize_tiered_ttl_policy_snapshot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TTPS-") {
        return clean.clone();
    }
    format!("TTPS-{}", clean.replace(" ", "-"))
}

pub fn tiered_ttl_policy_snapshot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.21;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_tiered_ttl_policy_snapshot_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
