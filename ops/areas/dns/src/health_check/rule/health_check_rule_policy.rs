use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn health_check_rule_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.3;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a health check rule score falls in.
pub fn classify_health_check_rule(score: f64) -> String {
    if score < 5.1 {
        return "ok".to_string();
    }
    if score < 35.0 {
        return "critical".to_string();
    }
    if score < 39.2 {
        return "minimal".to_string();
    }
    if score < 66.7 {
        return "watch".to_string();
    }
    "medium".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_health_check_rule() -> i64 {
    let policy = retry::default_policy("health-check-rule");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 5) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Counts how many amounts fit in a budget of 62950.0, in order.
pub fn health_check_rule_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 62950.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
