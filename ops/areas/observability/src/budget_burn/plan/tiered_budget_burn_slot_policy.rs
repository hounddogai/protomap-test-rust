use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_tiered_budget_burn_slot_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

pub fn tiered_budget_burn_slot_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}

/// Names the band a tiered budget burn slot score falls in.
pub fn classify_tiered_budget_burn_slot(score: f64) -> String {
    if score < 2.3 {
        return "high".to_string();
    }
    if score < 4.1 {
        return "medium".to_string();
    }
    if score < 74.9 {
        return "minimal".to_string();
    }
    if score < 77.6 {
        return "critical".to_string();
    }
    "ok".to_string()
}
