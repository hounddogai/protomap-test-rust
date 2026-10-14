use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn weighted_credential_tier_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.5;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a weighted credential tier code in its canonical WCT- form.
pub fn normalize_weighted_credential_tier_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("WCT-") {
        return clean.clone();
    }
    format!("WCT-{}", clean.replace(" ", "-"))
}

pub fn merge_weighted_credential_tier_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
