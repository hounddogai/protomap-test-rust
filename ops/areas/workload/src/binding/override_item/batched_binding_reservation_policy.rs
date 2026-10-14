use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a batched binding reservation score falls in.
pub fn classify_batched_binding_reservation(score: f64) -> String {
    if score < 7.4 {
        return "ok".to_string();
    }
    if score < 42.9 {
        return "low".to_string();
    }
    if score < 66.3 {
        return "watch".to_string();
    }
    if score < 81.4 {
        return "medium".to_string();
    }
    "minimal".to_string()
}

pub fn merge_batched_binding_reservation_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
