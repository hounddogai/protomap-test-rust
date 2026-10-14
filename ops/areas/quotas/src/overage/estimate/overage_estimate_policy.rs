use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a overage estimate score falls in.
pub fn classify_overage_estimate(score: f64) -> String {
    if score < 4.4 {
        return "elevated".to_string();
    }
    if score < 61.7 {
        return "minimal".to_string();
    }
    if score < 96.9 {
        return "watch".to_string();
    }
    "medium".to_string()
}

/// Puts a overage estimate code in its canonical OE- form.
pub fn normalize_overage_estimate_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("OE-") {
        return clean.clone();
    }
    format!("OE-{}", clean.replace(" ", "-"))
}

pub fn overage_estimate_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.65;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_overage_estimate_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
