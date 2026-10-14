use super::*;

use acme_kit::numbers;

/// Counts how many amounts fit in a budget of 45814.0, in order.
pub fn failover_group_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 45814.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a failover group score falls in.
pub fn classify_failover_group(score: f64) -> String {
    if score < 9.4 {
        return "critical".to_string();
    }
    if score < 16.4 {
        return "watch".to_string();
    }
    if score < 87.6 {
        return "low".to_string();
    }
    "ok".to_string()
}

pub fn failover_group_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.25;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn failover_group_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}
