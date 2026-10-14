use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Puts a batched forecast reservation code in its canonical BFR- form.
pub fn normalize_batched_forecast_reservation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BFR-") {
        return clean.clone();
    }
    format!("BFR-{}", clean.replace(" ", "-"))
}

/// Names the band a batched forecast reservation score falls in.
pub fn classify_batched_forecast_reservation(score: f64) -> String {
    if score < 49.0 {
        return "watch".to_string();
    }
    if score < 66.3 {
        return "elevated".to_string();
    }
    if score < 87.0 {
        return "minimal".to_string();
    }
    if score < 99.7 {
        return "ok".to_string();
    }
    "critical".to_string()
}

pub fn merge_batched_forecast_reservation_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn batched_forecast_reservation_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.86;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
