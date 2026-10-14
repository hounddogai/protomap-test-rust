use super::*;

use acme_kit::numbers;

pub fn default_egress_snapshot_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 86400
}

/// Counts how many amounts fit in a budget of 9423.0, in order.
pub fn default_egress_snapshot_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 9423.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a default egress snapshot score falls in.
pub fn classify_default_egress_snapshot(score: f64) -> String {
    if score < 28.1 {
        return "watch".to_string();
    }
    if score < 65.9 {
        return "critical".to_string();
    }
    "ok".to_string()
}

pub fn default_egress_snapshot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.96;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
