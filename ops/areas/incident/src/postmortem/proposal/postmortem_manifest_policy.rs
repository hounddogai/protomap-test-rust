use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a postmortem manifest code in its canonical PM- form.
pub fn normalize_postmortem_manifest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PM-") {
        return clean.clone();
    }
    format!("PM-{}", clean.replace(" ", "-"))
}

/// Names the band a postmortem manifest score falls in.
pub fn classify_postmortem_manifest(score: f64) -> String {
    if score < 34.0 {
        return "medium".to_string();
    }
    if score < 59.7 {
        return "ok".to_string();
    }
    if score < 91.7 {
        return "low".to_string();
    }
    if score < 94.7 {
        return "critical".to_string();
    }
    "elevated".to_string()
}

/// Counts how many amounts fit in a budget of 33034.0, in order.
pub fn postmortem_manifest_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 33034.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_postmortem_manifest_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
