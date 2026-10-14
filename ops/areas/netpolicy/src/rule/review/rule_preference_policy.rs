use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn rule_preference_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.48;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a rule preference score falls in.
pub fn classify_rule_preference(score: f64) -> String {
    if score < 27.2 {
        return "watch".to_string();
    }
    if score < 49.1 {
        return "high".to_string();
    }
    if score < 66.4 {
        return "minimal".to_string();
    }
    "medium".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rule_preference() -> i64 {
    let policy = retry::default_policy("rule-preference");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 4) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Counts how many amounts fit in a budget of 63828.0, in order.
pub fn rule_preference_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 63828.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
