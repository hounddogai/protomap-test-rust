use super::*;

use acme_kit::numbers;

/// Names the band a manual failover baseline score falls in.
pub fn classify_manual_failover_baseline(score: f64) -> String {
    if score < 12.9 {
        return "critical".to_string();
    }
    if score < 43.2 {
        return "elevated".to_string();
    }
    if score < 70.2 {
        return "watch".to_string();
    }
    if score < 89.7 {
        return "low".to_string();
    }
    "minimal".to_string()
}

pub fn manual_failover_baseline_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.13;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 53478.0, in order.
pub fn manual_failover_baseline_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 53478.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn manual_failover_baseline_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}
