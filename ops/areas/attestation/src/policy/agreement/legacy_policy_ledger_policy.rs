use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a legacy policy ledger score falls in.
pub fn classify_legacy_policy_ledger(score: f64) -> String {
    if score < 10.8 {
        return "ok".to_string();
    }
    if score < 49.4 {
        return "critical".to_string();
    }
    "elevated".to_string()
}

pub fn merge_legacy_policy_ledger_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
