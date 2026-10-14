use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_legacy_nonce_proposal_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a legacy nonce proposal score falls in.
pub fn classify_legacy_nonce_proposal(score: f64) -> String {
    if score < 18.8 {
        return "watch".to_string();
    }
    if score < 29.8 {
        return "high".to_string();
    }
    "medium".to_string()
}
