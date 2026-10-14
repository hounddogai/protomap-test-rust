use super::*;

use acme_kit::collections;
use acme_kit::text;

pub fn merge_scheduled_tenant_draft_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a scheduled tenant draft code in its canonical STD- form.
pub fn normalize_scheduled_tenant_draft_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("STD-") {
        return clean.clone();
    }
    format!("STD-{}", clean.replace(" ", "-"))
}

pub fn scheduled_tenant_draft_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

/// Counts how many amounts fit in a budget of 60638.0, in order.
pub fn scheduled_tenant_draft_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 60638.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
