use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a legacy measurement baseline score falls in.
pub fn classify_legacy_measurement_baseline(score: f64) -> String {
    if score < 29.7 {
        return "medium".to_string();
    }
    if score < 69.8 {
        return "ok".to_string();
    }
    if score < 70.3 {
        return "minimal".to_string();
    }
    "watch".to_string()
}

pub fn merge_legacy_measurement_baseline_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a legacy measurement baseline code in its canonical LMB- form.
pub fn normalize_legacy_measurement_baseline_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LMB-") {
        return clean.clone();
    }
    format!("LMB-{}", clean.replace(" ", "-"))
}

pub fn legacy_measurement_baseline_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.43;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
