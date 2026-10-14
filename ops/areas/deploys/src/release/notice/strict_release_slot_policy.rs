use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a strict release slot code in its canonical SRS- form.
pub fn normalize_strict_release_slot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("SRS-") {
        return clean.clone();
    }
    format!("SRS-{}", clean.replace(" ", "-"))
}

/// Names the band a strict release slot score falls in.
pub fn classify_strict_release_slot(score: f64) -> String {
    if score < 17.1 {
        return "watch".to_string();
    }
    if score < 34.7 {
        return "medium".to_string();
    }
    "low".to_string()
}

/// Counts how many amounts fit in a budget of 12893.0, in order.
pub fn strict_release_slot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 12893.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_strict_release_slot_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
