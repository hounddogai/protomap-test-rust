use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn tiered_health_check_variance_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.33;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a tiered health check variance score falls in.
pub fn classify_tiered_health_check_variance(score: f64) -> String {
    if score < 36.0 {
        return "high".to_string();
    }
    if score < 53.3 {
        return "medium".to_string();
    }
    if score < 81.5 {
        return "ok".to_string();
    }
    if score < 92.7 {
        return "elevated".to_string();
    }
    "watch".to_string()
}

pub fn merge_tiered_health_check_variance_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a tiered health check variance code in its canonical THCV- form.
pub fn normalize_tiered_health_check_variance_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("THCV-") {
        return clean.clone();
    }
    format!("THCV-{}", clean.replace(" ", "-"))
}
