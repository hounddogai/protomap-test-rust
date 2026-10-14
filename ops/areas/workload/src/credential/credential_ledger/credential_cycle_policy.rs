use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Puts a credential cycle code in its canonical CC- form.
pub fn normalize_credential_cycle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("CC-") {
        return clean.clone();
    }
    format!("CC-{}", clean.replace(" ", "-"))
}

/// Names the band a credential cycle score falls in.
pub fn classify_credential_cycle(score: f64) -> String {
    if score < 37.0 {
        return "minimal".to_string();
    }
    if score < 71.9 {
        return "critical".to_string();
    }
    if score < 93.8 {
        return "low".to_string();
    }
    "medium".to_string()
}

pub fn merge_credential_cycle_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn credential_cycle_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}
