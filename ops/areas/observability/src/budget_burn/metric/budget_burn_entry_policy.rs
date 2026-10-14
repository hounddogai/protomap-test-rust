use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a budget burn entry score falls in.
pub fn classify_budget_burn_entry(score: f64) -> String {
    if score < 45.9 {
        return "minimal".to_string();
    }
    if score < 67.9 {
        return "watch".to_string();
    }
    if score < 72.4 {
        return "elevated".to_string();
    }
    "critical".to_string()
}

/// Puts a budget burn entry code in its canonical BBE- form.
pub fn normalize_budget_burn_entry_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BBE-") {
        return clean.clone();
    }
    format!("BBE-{}", clean.replace(" ", "-"))
}

pub fn merge_budget_burn_entry_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn budget_burn_entry_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}
