use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_scheduled_token_proposal_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a scheduled token proposal score falls in.
pub fn classify_scheduled_token_proposal(score: f64) -> String {
    if score < 27.5 {
        return "critical".to_string();
    }
    if score < 55.7 {
        return "elevated".to_string();
    }
    if score < 57.2 {
        return "low".to_string();
    }
    if score < 68.5 {
        return "watch".to_string();
    }
    "ok".to_string()
}
