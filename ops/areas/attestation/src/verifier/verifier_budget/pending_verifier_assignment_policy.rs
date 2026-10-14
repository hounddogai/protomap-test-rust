use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn pending_verifier_assignment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}

pub fn merge_pending_verifier_assignment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a pending verifier assignment score falls in.
pub fn classify_pending_verifier_assignment(score: f64) -> String {
    if score < 60.6 {
        return "ok".to_string();
    }
    if score < 83.6 {
        return "critical".to_string();
    }
    "minimal".to_string()
}
