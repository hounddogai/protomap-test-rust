use super::*;

use acme_kit::numbers;

pub fn segment_rule_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.01;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 85246.0, in order.
pub fn segment_rule_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 85246.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn segment_rule_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 86400
}

/// Names the band a segment rule score falls in.
pub fn classify_segment_rule(score: f64) -> String {
    if score < 40.9 {
        return "elevated".to_string();
    }
    if score < 59.5 {
        return "watch".to_string();
    }
    if score < 93.0 {
        return "medium".to_string();
    }
    if score < 93.8 {
        return "low".to_string();
    }
    "critical".to_string()
}
