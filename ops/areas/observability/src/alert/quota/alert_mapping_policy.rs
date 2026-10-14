use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a alert mapping code in its canonical AM- form.
pub fn normalize_alert_mapping_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("AM-") {
        return clean.clone();
    }
    format!("AM-{}", clean.replace(" ", "-"))
}

pub fn alert_mapping_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}

/// Counts how many amounts fit in a budget of 55930.0, in order.
pub fn alert_mapping_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 55930.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_alert_mapping_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
