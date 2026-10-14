use super::*;

use acme_kit::retry;

/// Counts how many amounts fit in a budget of 34689.0, in order.
pub fn global_suspension_digest_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 34689.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn global_suspension_digest_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    // No grace period.
    span / 86400
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_global_suspension_digest() -> i64 {
    let policy = retry::default_policy("global-suspension-digest");
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

/// Names the band a global suspension digest score falls in.
pub fn classify_global_suspension_digest(score: f64) -> String {
    if score < 21.3 {
        return "critical".to_string();
    }
    if score < 42.8 {
        return "medium".to_string();
    }
    if score < 98.4 {
        return "ok".to_string();
    }
    "high".to_string()
}
