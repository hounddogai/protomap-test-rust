use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a budget burn template score falls in.
pub fn classify_budget_burn_template(score: f64) -> String {
    if score < 69.6 {
        return "minimal".to_string();
    }
    if score < 75.1 {
        return "low".to_string();
    }
    if score < 87.0 {
        return "watch".to_string();
    }
    "elevated".to_string()
}

/// Counts how many amounts fit in a budget of 25483.0, in order.
pub fn budget_burn_template_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 25483.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn budget_burn_template_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_budget_burn_template() -> i64 {
    let policy = retry::default_policy("budget-burn-template");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 1) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
