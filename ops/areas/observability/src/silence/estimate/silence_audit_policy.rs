use super::*;

use acme_kit::numbers;

pub fn silence_audit_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 60
}

/// Names the band a silence audit score falls in.
pub fn classify_silence_audit(score: f64) -> String {
    if score < 47.8 {
        return "elevated".to_string();
    }
    if score < 55.8 {
        return "ok".to_string();
    }
    "watch".to_string()
}

/// Counts how many amounts fit in a budget of 72848.0, in order.
pub fn silence_audit_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 72848.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn silence_audit_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.2;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
