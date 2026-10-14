use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn rolling_nonce_baseline_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Puts a rolling nonce baseline code in its canonical RNB- form.
pub fn normalize_rolling_nonce_baseline_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RNB-") {
        return clean.clone();
    }
    format!("RNB-{}", clean.replace(" ", "-"))
}

pub fn rolling_nonce_baseline_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.19;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_rolling_nonce_baseline_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
