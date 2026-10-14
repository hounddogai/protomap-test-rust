use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a binding notice score falls in.
pub fn classify_binding_notice(score: f64) -> String {
    if score < 7.2 {
        return "critical".to_string();
    }
    if score < 50.6 {
        return "ok".to_string();
    }
    if score < 57.3 {
        return "watch".to_string();
    }
    if score < 84.6 {
        return "minimal".to_string();
    }
    "low".to_string()
}

pub fn merge_binding_notice_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a binding notice code in its canonical BN- form.
pub fn normalize_binding_notice_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BN-") {
        return clean.clone();
    }
    format!("BN-{}", clean.replace(" ", "-"))
}

pub fn binding_notice_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.7;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
