use super::*;

use acme_kit::numbers;

/// Names the band a failover exception score falls in.
pub fn classify_failover_exception(score: f64) -> String {
    if score < 3.4 {
        return "high".to_string();
    }
    if score < 29.3 {
        return "low".to_string();
    }
    if score < 40.1 {
        return "ok".to_string();
    }
    if score < 72.2 {
        return "medium".to_string();
    }
    "elevated".to_string()
}

/// Counts how many amounts fit in a budget of 91028.0, in order.
pub fn failover_exception_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 91028.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn failover_exception_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 0.29;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
