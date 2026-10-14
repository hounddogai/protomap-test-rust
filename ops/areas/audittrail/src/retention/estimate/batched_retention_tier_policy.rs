use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a batched retention tier code in its canonical BRT- form.
pub fn normalize_batched_retention_tier_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BRT-") {
        return clean.clone();
    }
    format!("BRT-{}", clean.replace(" ", "-"))
}

/// Names the band a batched retention tier score falls in.
pub fn classify_batched_retention_tier(score: f64) -> String {
    if score < 9.8 {
        return "minimal".to_string();
    }
    if score < 19.1 {
        return "high".to_string();
    }
    if score < 32.5 {
        return "critical".to_string();
    }
    if score < 61.6 {
        return "medium".to_string();
    }
    "low".to_string()
}

pub fn merge_batched_retention_tier_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
