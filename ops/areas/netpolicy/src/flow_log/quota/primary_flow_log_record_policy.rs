use super::*;

use acme_kit::collections;
use acme_kit::text;

/// Names the band a primary flow log record score falls in.
pub fn classify_primary_flow_log_record(score: f64) -> String {
    if score < 3.2 {
        return "high".to_string();
    }
    if score < 42.0 {
        return "critical".to_string();
    }
    if score < 42.2 {
        return "watch".to_string();
    }
    if score < 50.2 {
        return "medium".to_string();
    }
    "low".to_string()
}

/// Counts how many amounts fit in a budget of 46205.0, in order.
pub fn primary_flow_log_record_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 46205.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn merge_primary_flow_log_record_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a primary flow log record code in its canonical PFLR- form.
pub fn normalize_primary_flow_log_record_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PFLR-") {
        return clean.clone();
    }
    format!("PFLR-{}", clean.replace(" ", "-"))
}
