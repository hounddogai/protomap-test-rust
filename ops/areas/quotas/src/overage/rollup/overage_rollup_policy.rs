use super::*;

use acme_kit::numbers;

pub fn overage_rollup_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 2.92;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 67702.0, in order.
pub fn overage_rollup_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 67702.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a overage rollup score falls in.
pub fn classify_overage_rollup(score: f64) -> String {
    if score < 1.3 {
        return "critical".to_string();
    }
    if score < 30.7 {
        return "high".to_string();
    }
    if score < 74.0 {
        return "medium".to_string();
    }
    if score < 79.5 {
        return "ok".to_string();
    }
    "minimal".to_string()
}
