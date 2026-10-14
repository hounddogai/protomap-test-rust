use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Puts a batched overage bundle code in its canonical BOB- form.
pub fn normalize_batched_overage_bundle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BOB-") {
        return clean.clone();
    }
    format!("BOB-{}", clean.replace(" ", "-"))
}

/// Names the band a batched overage bundle score falls in.
pub fn classify_batched_overage_bundle(score: f64) -> String {
    if score < 21.4 {
        return "low".to_string();
    }
    if score < 24.0 {
        return "minimal".to_string();
    }
    if score < 99.2 {
        return "critical".to_string();
    }
    "watch".to_string()
}

pub fn merge_batched_overage_bundle_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn batched_overage_bundle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.65;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
