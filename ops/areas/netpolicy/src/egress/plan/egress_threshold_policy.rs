use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a egress threshold score falls in.
pub fn classify_egress_threshold(score: f64) -> String {
    if score < 46.4 {
        return "ok".to_string();
    }
    if score < 65.6 {
        return "elevated".to_string();
    }
    if score < 93.2 {
        return "watch".to_string();
    }
    "minimal".to_string()
}

/// Puts a egress threshold code in its canonical ET- form.
pub fn normalize_egress_threshold_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ET-") {
        return clean.clone();
    }
    format!("ET-{}", clean.replace(" ", "-"))
}

pub fn merge_egress_threshold_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
