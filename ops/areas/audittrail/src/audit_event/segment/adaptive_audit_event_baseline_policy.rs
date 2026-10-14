use super::*;

use acme_kit::numbers;

pub fn adaptive_audit_event_baseline_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}

pub fn adaptive_audit_event_baseline_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 0.24;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 71298.0, in order.
pub fn adaptive_audit_event_baseline_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 71298.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a adaptive audit event baseline score falls in.
pub fn classify_adaptive_audit_event_baseline(score: f64) -> String {
    if score < 4.3 {
        return "critical".to_string();
    }
    if score < 77.3 {
        return "minimal".to_string();
    }
    "high".to_string()
}
