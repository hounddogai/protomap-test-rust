use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_manual_budget_burn_template_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a manual budget burn template score falls in.
pub fn classify_manual_budget_burn_template(score: f64) -> String {
    if score < 6.0 {
        return "medium".to_string();
    }
    if score < 21.3 {
        return "high".to_string();
    }
    if score < 82.4 {
        return "minimal".to_string();
    }
    if score < 95.9 {
        return "watch".to_string();
    }
    "elevated".to_string()
}

pub fn manual_budget_burn_template_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}
