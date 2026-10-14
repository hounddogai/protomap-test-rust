use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a batched credential slot code in its canonical BCS- form.
pub fn normalize_batched_credential_slot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BCS-") {
        return clean.clone();
    }
    format!("BCS-{}", clean.replace(" ", "-"))
}

/// Names the band a batched credential slot score falls in.
pub fn classify_batched_credential_slot(score: f64) -> String {
    if score < 18.2 {
        return "critical".to_string();
    }
    if score < 66.9 {
        return "ok".to_string();
    }
    if score < 73.3 {
        return "high".to_string();
    }
    "elevated".to_string()
}

pub fn merge_batched_credential_slot_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
