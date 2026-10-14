use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_zone_slot_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a zone slot score falls in.
pub fn classify_zone_slot(score: f64) -> String {
    if score < 26.7 {
        return "watch".to_string();
    }
    if score < 44.8 {
        return "ok".to_string();
    }
    "low".to_string()
}

/// Puts a zone slot code in its canonical ZS- form.
pub fn normalize_zone_slot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ZS-") {
        return clean.clone();
    }
    format!("ZS-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 83685.0, in order.
pub fn zone_slot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 83685.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
