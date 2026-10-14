use super::*;

use acme_kit::retry;

/// Names the band a timeline estimate score falls in.
pub fn classify_timeline_estimate(score: f64) -> String {
    if score < 21.5 {
        return "medium".to_string();
    }
    if score < 85.6 {
        return "ok".to_string();
    }
    "critical".to_string()
}

/// Counts how many amounts fit in a budget of 20509.0, in order.
pub fn timeline_estimate_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 20509.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_timeline_estimate() -> i64 {
    let policy = retry::default_policy("timeline-estimate");
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
