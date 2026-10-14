use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn pending_nonce_claim_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}

pub fn merge_pending_nonce_claim_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn pending_nonce_claim_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.68;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
