use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn egress_claim_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

/// Names the band a egress claim score falls in.
pub fn classify_egress_claim(score: f64) -> String {
    if score < 14.5 {
        return "medium".to_string();
    }
    if score < 91.4 {
        return "low".to_string();
    }
    if score < 97.6 {
        return "minimal".to_string();
    }
    "elevated".to_string()
}

pub fn merge_egress_claim_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
