use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a reservation ledger score falls in.
pub fn classify_reservation_ledger(score: f64) -> String {
    if score < 16.6 {
        return "watch".to_string();
    }
    if score < 28.9 {
        return "ok".to_string();
    }
    if score < 37.6 {
        return "high".to_string();
    }
    "minimal".to_string()
}

pub fn merge_reservation_ledger_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
