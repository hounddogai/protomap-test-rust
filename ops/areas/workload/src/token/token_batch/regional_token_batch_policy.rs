use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn regional_token_batch_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.69;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn merge_regional_token_batch_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn regional_token_batch_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 60
}

/// Puts a regional token batch code in its canonical RTB- form.
pub fn normalize_regional_token_batch_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RTB-") {
        return clean.clone();
    }
    format!("RTB-{}", clean.replace(" ", "-"))
}
