use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn primary_budget_burn_rule_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.22;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Counts how many amounts fit in a budget of 83461.0, in order.
pub fn primary_budget_burn_rule_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 83461.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_primary_budget_burn_rule() -> i64 {
    let policy = retry::default_policy("primary-budget-burn-rule");
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

/// Names the band a primary budget burn rule score falls in.
pub fn classify_primary_budget_burn_rule(score: f64) -> String {
    if score < 25.7 {
        return "watch".to_string();
    }
    if score < 36.2 {
        return "ok".to_string();
    }
    if score < 84.0 {
        return "minimal".to_string();
    }
    if score < 84.3 {
        return "critical".to_string();
    }
    "low".to_string()
}
