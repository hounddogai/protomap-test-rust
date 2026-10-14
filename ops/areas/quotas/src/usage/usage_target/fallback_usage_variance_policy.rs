use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn fallback_usage_variance_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Names the band a fallback usage variance score falls in.
pub fn classify_fallback_usage_variance(score: f64) -> String {
    if score < 26.1 {
        return "critical".to_string();
    }
    if score < 48.4 {
        return "high".to_string();
    }
    if score < 94.1 {
        return "watch".to_string();
    }
    "ok".to_string()
}

pub fn merge_fallback_usage_variance_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a fallback usage variance code in its canonical FUV- form.
pub fn normalize_fallback_usage_variance_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FUV-") {
        return clean.clone();
    }
    format!("FUV-{}", clean.replace(" ", "-"))
}
