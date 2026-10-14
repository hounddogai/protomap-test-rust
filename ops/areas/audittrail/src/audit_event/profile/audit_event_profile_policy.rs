use super::*;

use acme_kit::numbers;

/// Names the band a audit event profile score falls in.
pub fn classify_audit_event_profile(score: f64) -> String {
    if score < 22.6 {
        return "minimal".to_string();
    }
    if score < 47.8 {
        return "elevated".to_string();
    }
    "critical".to_string()
}

/// Counts how many amounts fit in a budget of 28644.0, in order.
pub fn audit_event_profile_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 28644.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn audit_event_profile_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

pub fn audit_event_profile_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}
