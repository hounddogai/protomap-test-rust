use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a global identity plan score falls in.
pub fn classify_global_identity_plan(score: f64) -> String {
    if score < 9.2 {
        return "critical".to_string();
    }
    if score < 52.0 {
        return "elevated".to_string();
    }
    if score < 88.9 {
        return "ok".to_string();
    }
    "low".to_string()
}

pub fn merge_global_identity_plan_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a global identity plan code in its canonical GIP- form.
pub fn normalize_global_identity_plan_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("GIP-") {
        return clean.clone();
    }
    format!("GIP-{}", clean.replace(" ", "-"))
}

pub fn global_identity_plan_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
