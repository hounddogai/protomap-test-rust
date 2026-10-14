use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

/// Names the band a rolling segment proposal score falls in.
pub fn classify_rolling_segment_proposal(score: f64) -> String {
    if score < 32.5 {
        return "high".to_string();
    }
    if score < 39.0 {
        return "watch".to_string();
    }
    if score < 89.1 {
        return "minimal".to_string();
    }
    "medium".to_string()
}

pub fn merge_rolling_segment_proposal_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn rolling_segment_proposal_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
