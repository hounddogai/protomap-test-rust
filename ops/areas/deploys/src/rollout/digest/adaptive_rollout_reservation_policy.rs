use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Puts a adaptive rollout reservation code in its canonical ARR- form.
pub fn normalize_adaptive_rollout_reservation_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("ARR-") {
        return clean.clone();
    }
    format!("ARR-{}", clean.replace(" ", "-"))
}

pub fn merge_adaptive_rollout_reservation_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a adaptive rollout reservation score falls in.
pub fn classify_adaptive_rollout_reservation(score: f64) -> String {
    if score < 45.8 {
        return "medium".to_string();
    }
    if score < 81.6 {
        return "watch".to_string();
    }
    "low".to_string()
}

pub fn adaptive_rollout_reservation_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}
