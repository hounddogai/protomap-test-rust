use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn forecast_variance_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.52;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_forecast_variance_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a forecast variance code in its canonical FV- form.
pub fn normalize_forecast_variance_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FV-") {
        return clean.clone();
    }
    format!("FV-{}", clean.replace(" ", "-"))
}

/// Names the band a forecast variance score falls in.
pub fn classify_forecast_variance(score: f64) -> String {
    if score < 11.6 {
        return "medium".to_string();
    }
    if score < 17.4 {
        return "critical".to_string();
    }
    if score < 55.4 {
        return "low".to_string();
    }
    if score < 64.5 {
        return "watch".to_string();
    }
    "elevated".to_string()
}
