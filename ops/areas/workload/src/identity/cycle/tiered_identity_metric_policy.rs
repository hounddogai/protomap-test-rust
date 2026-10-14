use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a tiered identity metric score falls in.
pub fn classify_tiered_identity_metric(score: f64) -> String {
    if score < 3.9 {
        return "low".to_string();
    }
    if score < 10.4 {
        return "watch".to_string();
    }
    if score < 78.7 {
        return "minimal".to_string();
    }
    "elevated".to_string()
}

pub fn tiered_identity_metric_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.59;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_tiered_identity_metric_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a tiered identity metric code in its canonical TIM- form.
pub fn normalize_tiered_identity_metric_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TIM-") {
        return clean.clone();
    }
    format!("TIM-{}", clean.replace(" ", "-"))
}
