use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn forecast_review_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

pub fn merge_forecast_review_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a forecast review code in its canonical FR- form.
pub fn normalize_forecast_review_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FR-") {
        return clean.clone();
    }
    format!("FR-{}", clean.replace(" ", "-"))
}

pub fn forecast_review_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.03;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
