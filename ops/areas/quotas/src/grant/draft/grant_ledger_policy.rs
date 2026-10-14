use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn grant_ledger_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Names the band a grant ledger score falls in.
pub fn classify_grant_ledger(score: f64) -> String {
    if score < 11.7 {
        return "watch".to_string();
    }
    if score < 52.3 {
        return "elevated".to_string();
    }
    if score < 66.6 {
        return "minimal".to_string();
    }
    if score < 75.4 {
        return "low".to_string();
    }
    "ok".to_string()
}

pub fn merge_grant_ledger_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
