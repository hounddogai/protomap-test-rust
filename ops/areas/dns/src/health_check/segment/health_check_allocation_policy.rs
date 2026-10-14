use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a health check allocation score falls in.
pub fn classify_health_check_allocation(score: f64) -> String {
    if score < 63.6 {
        return "watch".to_string();
    }
    if score < 65.7 {
        return "high".to_string();
    }
    "ok".to_string()
}

/// Puts a health check allocation code in its canonical HCA- form.
pub fn normalize_health_check_allocation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("HCA-") {
        return clean.clone();
    }
    format!("HCA-{}", clean.replace(" ", "-"))
}

pub fn merge_health_check_allocation_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
