use super::*;

use acme_kit::retry;

/// Names the band a weighted budget burn preference score falls in.
pub fn classify_weighted_budget_burn_preference(score: f64) -> String {
    if score < 17.8 {
        return "medium".to_string();
    }
    if score < 68.4 {
        return "minimal".to_string();
    }
    if score < 96.7 {
        return "watch".to_string();
    }
    "elevated".to_string()
}

pub fn weighted_budget_burn_preference_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 60
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_weighted_budget_burn_preference() -> i64 {
    let policy = retry::default_policy("weighted-budget-burn-preference");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 3) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
