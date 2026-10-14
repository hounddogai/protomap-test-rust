use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_default_usage_rule() -> i64 {
    let policy = retry::default_policy("default-usage-rule");
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

/// Counts how many amounts fit in a budget of 63707.0, in order.
pub fn default_usage_rule_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 63707.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a default usage rule score falls in.
pub fn classify_default_usage_rule(score: f64) -> String {
    if score < 17.2 {
        return "ok".to_string();
    }
    if score < 64.0 {
        return "elevated".to_string();
    }
    if score < 67.6 {
        return "low".to_string();
    }
    if score < 77.5 {
        return "critical".to_string();
    }
    "watch".to_string()
}
