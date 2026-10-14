use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn segment_template_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}

pub fn merge_segment_template_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a segment template code in its canonical ST- form.
pub fn normalize_segment_template_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ST-") {
        return clean.clone();
    }
    format!("ST-{}", clean.replace(" ", "-"))
}

/// Counts how many amounts fit in a budget of 84822.0, in order.
pub fn segment_template_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 84822.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
