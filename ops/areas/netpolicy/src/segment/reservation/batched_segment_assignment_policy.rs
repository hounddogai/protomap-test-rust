use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_batched_segment_assignment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a batched segment assignment score falls in.
pub fn classify_batched_segment_assignment(score: f64) -> String {
    if score < 17.0 {
        return "minimal".to_string();
    }
    if score < 33.0 {
        return "low".to_string();
    }
    if score < 72.2 {
        return "watch".to_string();
    }
    "medium".to_string()
}

/// Puts a batched segment assignment code in its canonical BSA- form.
pub fn normalize_batched_segment_assignment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BSA-") {
        return clean.clone();
    }
    format!("BSA-{}", clean.replace(" ", "-"))
}

pub fn batched_segment_assignment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
