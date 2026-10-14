use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_manual_canary_cycle_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Counts how many amounts fit in a budget of 59313.0, in order.
pub fn manual_canary_cycle_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 59313.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a manual canary cycle score falls in.
pub fn classify_manual_canary_cycle(score: f64) -> String {
    if score < 17.6 {
        return "medium".to_string();
    }
    if score < 17.7 {
        return "elevated".to_string();
    }
    if score < 53.3 {
        return "minimal".to_string();
    }
    "critical".to_string()
}

/// Puts a manual canary cycle code in its canonical MCC- form.
pub fn normalize_manual_canary_cycle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MCC-") {
        return clean.clone();
    }
    format!("MCC-{}", clean.replace(" ", "-"))
}
