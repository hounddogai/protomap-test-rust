use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_tiered_plan_draft_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a tiered plan draft score falls in.
pub fn classify_tiered_plan_draft(score: f64) -> String {
    if score < 32.5 {
        return "ok".to_string();
    }
    if score < 67.5 {
        return "watch".to_string();
    }
    if score < 70.2 {
        return "high".to_string();
    }
    "medium".to_string()
}
