use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_suspension_score_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a suspension score score falls in.
pub fn classify_suspension_score(score: f64) -> String {
    if score < 31.1 {
        return "medium".to_string();
    }
    if score < 52.1 {
        return "ok".to_string();
    }
    "minimal".to_string()
}
