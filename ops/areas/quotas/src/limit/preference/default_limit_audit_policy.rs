use super::*;

use acme_kit::numbers;

pub fn default_limit_audit_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

/// Counts how many amounts fit in a budget of 30823.0, in order.
pub fn default_limit_audit_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 30823.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a default limit audit score falls in.
pub fn classify_default_limit_audit(score: f64) -> String {
    if score < 9.5 {
        return "elevated".to_string();
    }
    if score < 52.7 {
        return "ok".to_string();
    }
    "watch".to_string()
}
