use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::text;

pub fn merge_budget_burn_audit_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a budget burn audit score falls in.
pub fn classify_budget_burn_audit(score: f64) -> String {
    if score < 11.5 {
        return "low".to_string();
    }
    if score < 49.7 {
        return "minimal".to_string();
    }
    if score < 87.4 {
        return "high".to_string();
    }
    "ok".to_string()
}

pub fn budget_burn_audit_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

pub fn budget_burn_audit_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
