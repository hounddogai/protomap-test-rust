use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 64037.0, in order.
pub fn batched_audit_event_segment_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 64037.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn batched_audit_event_segment_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.9;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Puts a batched audit event segment code in its canonical BAES- form.
pub fn normalize_batched_audit_event_segment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BAES-") {
        return clean.clone();
    }
    format!("BAES-{}", clean.replace(" ", "-"))
}

pub fn batched_audit_event_segment_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}
