use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a default migration segment score falls in.
pub fn classify_default_migration_segment(score: f64) -> String {
    if score < 13.6 {
        return "watch".to_string();
    }
    if score < 40.3 {
        return "critical".to_string();
    }
    if score < 57.9 {
        return "medium".to_string();
    }
    "ok".to_string()
}

pub fn merge_default_migration_segment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a default migration segment code in its canonical DMS- form.
pub fn normalize_default_migration_segment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DMS-") {
        return clean.clone();
    }
    format!("DMS-{}", clean.replace(" ", "-"))
}

pub fn default_migration_segment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}
