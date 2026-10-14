use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn canary_variance_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.76;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a canary variance score falls in.
pub fn classify_canary_variance(score: f64) -> String {
    if score < 36.6 {
        return "ok".to_string();
    }
    if score < 37.5 {
        return "critical".to_string();
    }
    if score < 43.0 {
        return "minimal".to_string();
    }
    if score < 90.8 {
        return "high".to_string();
    }
    "medium".to_string()
}

/// Puts a canary variance code in its canonical CV- form.
pub fn normalize_canary_variance_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CV-") {
        return clean.clone();
    }
    format!("CV-{}", clean.replace(" ", "-"))
}

pub fn merge_canary_variance_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
