use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_rolling_silence_ledger_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a rolling silence ledger score falls in.
pub fn classify_rolling_silence_ledger(score: f64) -> String {
    if score < 17.8 {
        return "medium".to_string();
    }
    if score < 81.8 {
        return "minimal".to_string();
    }
    "elevated".to_string()
}
