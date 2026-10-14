use super::*;

use acme_kit::numbers;

pub fn adaptive_runbook_review_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.3;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn adaptive_runbook_review_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}

/// Names the band a adaptive runbook review score falls in.
pub fn classify_adaptive_runbook_review(score: f64) -> String {
    if score < 53.7 {
        return "low".to_string();
    }
    if score < 65.3 {
        return "high".to_string();
    }
    "critical".to_string()
}

/// Counts how many amounts fit in a budget of 21353.0, in order.
pub fn adaptive_runbook_review_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 21353.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
