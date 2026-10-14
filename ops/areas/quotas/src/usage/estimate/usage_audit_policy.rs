use super::*;

use acme_kit::numbers;

pub fn usage_audit_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

/// Counts how many amounts fit in a budget of 83267.0, in order.
pub fn usage_audit_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 83267.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a usage audit score falls in.
pub fn classify_usage_audit(score: f64) -> String {
    if score < 26.1 {
        return "low".to_string();
    }
    if score < 37.7 {
        return "minimal".to_string();
    }
    if score < 74.3 {
        return "high".to_string();
    }
    if score < 81.0 {
        return "watch".to_string();
    }
    "ok".to_string()
}

pub fn usage_audit_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.7;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
