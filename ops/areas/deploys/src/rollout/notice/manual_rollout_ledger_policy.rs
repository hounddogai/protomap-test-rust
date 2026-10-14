use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_manual_rollout_ledger_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a manual rollout ledger score falls in.
pub fn classify_manual_rollout_ledger(score: f64) -> String {
    if score < 3.0 {
        return "watch".to_string();
    }
    if score < 16.1 {
        return "low".to_string();
    }
    if score < 69.3 {
        return "critical".to_string();
    }
    "minimal".to_string()
}

pub fn manual_rollout_ledger_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}
