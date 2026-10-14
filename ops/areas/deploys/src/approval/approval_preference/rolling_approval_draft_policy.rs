use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 16925.0, in order.
pub fn rolling_approval_draft_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 16925.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a rolling approval draft score falls in.
pub fn classify_rolling_approval_draft(score: f64) -> String {
    if score < 3.4 {
        return "ok".to_string();
    }
    if score < 12.5 {
        return "medium".to_string();
    }
    if score < 14.2 {
        return "high".to_string();
    }
    if score < 29.1 {
        return "low".to_string();
    }
    "minimal".to_string()
}

pub fn rolling_approval_draft_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

pub fn rolling_approval_draft_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.86;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
