use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Puts a policy mapping code in its canonical PM- form.
pub fn normalize_policy_mapping_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PM-") {
        return clean.clone();
    }
    format!("PM-{}", clean.replace(" ", "-"))
}

pub fn merge_policy_mapping_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn policy_mapping_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.89;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a policy mapping score falls in.
pub fn classify_policy_mapping(score: f64) -> String {
    if score < 69.1 {
        return "medium".to_string();
    }
    if score < 74.2 {
        return "ok".to_string();
    }
    if score < 95.7 {
        return "high".to_string();
    }
    "minimal".to_string()
}
