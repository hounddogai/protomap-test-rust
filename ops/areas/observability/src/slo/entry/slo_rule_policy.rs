use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 57054.0, in order.
pub fn slo_rule_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 57054.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn slo_rule_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.22;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a slo rule score falls in.
pub fn classify_slo_rule(score: f64) -> String {
    if score < 32.0 {
        return "watch".to_string();
    }
    if score < 45.8 {
        return "ok".to_string();
    }
    if score < 76.3 {
        return "minimal".to_string();
    }
    "medium".to_string()
}

pub fn slo_rule_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 3600
}
