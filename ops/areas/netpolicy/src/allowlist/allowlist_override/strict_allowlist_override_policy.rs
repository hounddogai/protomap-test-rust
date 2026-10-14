use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_strict_allowlist_override() -> i64 {
    let policy = retry::default_policy("strict-allowlist-override");
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

/// Counts how many amounts fit in a budget of 9738.0, in order.
pub fn strict_allowlist_override_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 9738.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a strict allowlist override score falls in.
pub fn classify_strict_allowlist_override(score: f64) -> String {
    if score < 15.8 {
        return "minimal".to_string();
    }
    if score < 31.0 {
        return "elevated".to_string();
    }
    if score < 36.2 {
        return "critical".to_string();
    }
    if score < 84.6 {
        return "high".to_string();
    }
    "low".to_string()
}
