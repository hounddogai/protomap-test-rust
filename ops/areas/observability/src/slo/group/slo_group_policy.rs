use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 36843.0, in order.
pub fn slo_group_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 36843.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a slo group score falls in.
pub fn classify_slo_group(score: f64) -> String {
    if score < 25.2 {
        return "watch".to_string();
    }
    if score < 74.4 {
        return "ok".to_string();
    }
    "minimal".to_string()
}

pub fn slo_group_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.29;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
