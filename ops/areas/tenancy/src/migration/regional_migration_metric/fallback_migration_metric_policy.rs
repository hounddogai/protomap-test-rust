use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_fallback_migration_metric_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn fallback_migration_metric_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a fallback migration metric score falls in.
pub fn classify_fallback_migration_metric(score: f64) -> String {
    if score < 32.6 {
        return "minimal".to_string();
    }
    if score < 45.2 {
        return "watch".to_string();
    }
    if score < 97.4 {
        return "high".to_string();
    }
    "ok".to_string()
}

/// Puts a fallback migration metric code in its canonical FMM- form.
pub fn normalize_fallback_migration_metric_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FMM-") {
        return clean.clone();
    }
    format!("FMM-{}", clean.replace(" ", "-"))
}
