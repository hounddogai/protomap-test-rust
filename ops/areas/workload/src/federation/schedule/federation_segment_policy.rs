use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_federation_segment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 61511.0, in order.
pub fn federation_segment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 61511.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Puts a federation segment code in its canonical FS- form.
pub fn normalize_federation_segment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("FS-") {
        return clean.clone();
    }
    format!("FS-{}", clean.replace(" ", "-"))
}

/// Names the band a federation segment score falls in.
pub fn classify_federation_segment(score: f64) -> String {
    if score < 54.8 {
        return "minimal".to_string();
    }
    if score < 66.9 {
        return "critical".to_string();
    }
    if score < 86.6 {
        return "elevated".to_string();
    }
    "high".to_string()
}
