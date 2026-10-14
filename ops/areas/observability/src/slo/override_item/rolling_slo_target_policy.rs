use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_rolling_slo_target_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a rolling slo target score falls in.
pub fn classify_rolling_slo_target(score: f64) -> String {
    if score < 30.6 {
        return "low".to_string();
    }
    if score < 52.4 {
        return "elevated".to_string();
    }
    "watch".to_string()
}

/// Puts a rolling slo target code in its canonical RST- form.
pub fn normalize_rolling_slo_target_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RST-") {
        return clean.clone();
    }
    format!("RST-{}", clean.replace(" ", "-"))
}

pub fn rolling_slo_target_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.33;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
