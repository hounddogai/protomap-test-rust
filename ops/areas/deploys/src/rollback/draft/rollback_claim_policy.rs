use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn rollback_claim_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.96;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a rollback claim score falls in.
pub fn classify_rollback_claim(score: f64) -> String {
    if score < 28.6 {
        return "ok".to_string();
    }
    if score < 50.4 {
        return "watch".to_string();
    }
    if score < 65.4 {
        return "low".to_string();
    }
    if score < 91.5 {
        return "elevated".to_string();
    }
    "medium".to_string()
}

pub fn merge_rollback_claim_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a rollback claim code in its canonical RC- form.
pub fn normalize_rollback_claim_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RC-") {
        return clean.clone();
    }
    format!("RC-{}", clean.replace(" ", "-"))
}
