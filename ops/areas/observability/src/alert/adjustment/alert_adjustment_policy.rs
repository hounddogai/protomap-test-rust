use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Puts a alert adjustment code in its canonical AA- form.
pub fn normalize_alert_adjustment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AA-") {
        return clean.clone();
    }
    format!("AA-{}", clean.replace(" ", "-"))
}

/// Names the band a alert adjustment score falls in.
pub fn classify_alert_adjustment(score: f64) -> String {
    if score < 66.9 {
        return "low".to_string();
    }
    if score < 79.1 {
        return "ok".to_string();
    }
    if score < 84.6 {
        return "minimal".to_string();
    }
    if score < 97.2 {
        return "critical".to_string();
    }
    "watch".to_string()
}

pub fn alert_adjustment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.21;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_alert_adjustment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
