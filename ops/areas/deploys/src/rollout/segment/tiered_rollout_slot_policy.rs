use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn tiered_rollout_slot_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Puts a tiered rollout slot code in its canonical TRS- form.
pub fn normalize_tiered_rollout_slot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TRS-") {
        return clean.clone();
    }
    format!("TRS-{}", clean.replace(" ", "-"))
}

/// Names the band a tiered rollout slot score falls in.
pub fn classify_tiered_rollout_slot(score: f64) -> String {
    if score < 30.0 {
        return "medium".to_string();
    }
    if score < 35.9 {
        return "elevated".to_string();
    }
    if score < 69.2 {
        return "ok".to_string();
    }
    "minimal".to_string()
}

pub fn merge_tiered_rollout_slot_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
