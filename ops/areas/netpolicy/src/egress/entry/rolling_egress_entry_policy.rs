use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn rolling_egress_entry_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.04;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a rolling egress entry score falls in.
pub fn classify_rolling_egress_entry(score: f64) -> String {
    if score < 14.0 {
        return "elevated".to_string();
    }
    if score < 19.0 {
        return "watch".to_string();
    }
    if score < 48.4 {
        return "low".to_string();
    }
    if score < 91.2 {
        return "ok".to_string();
    }
    "medium".to_string()
}

/// Puts a rolling egress entry code in its canonical REE- form.
pub fn normalize_rolling_egress_entry_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("REE-") {
        return clean.clone();
    }
    format!("REE-{}", clean.replace(" ", "-"))
}

pub fn merge_rolling_egress_entry_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
