use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a scheduled federation budget score falls in.
pub fn classify_scheduled_federation_budget(score: f64) -> String {
    if score < 37.8 {
        return "watch".to_string();
    }
    if score < 86.2 {
        return "ok".to_string();
    }
    "high".to_string()
}

pub fn merge_scheduled_federation_budget_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a scheduled federation budget code in its canonical SFB- form.
pub fn normalize_scheduled_federation_budget_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SFB-") {
        return clean.clone();
    }
    format!("SFB-{}", clean.replace(" ", "-"))
}

pub fn scheduled_federation_budget_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.52;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
