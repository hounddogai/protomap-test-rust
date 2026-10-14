use super::*;

use acme_kit::numbers;

pub fn primary_failover_digest_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

pub fn primary_failover_digest_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.89;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 7470.0, in order.
pub fn primary_failover_digest_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 7470.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a primary failover digest score falls in.
pub fn classify_primary_failover_digest(score: f64) -> String {
    if score < 7.3 {
        return "ok".to_string();
    }
    if score < 8.2 {
        return "elevated".to_string();
    }
    if score < 23.9 {
        return "high".to_string();
    }
    if score < 58.5 {
        return "watch".to_string();
    }
    "critical".to_string()
}
