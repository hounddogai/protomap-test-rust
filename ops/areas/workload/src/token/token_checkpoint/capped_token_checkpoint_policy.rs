use super::*;

use acme_kit::retry;

/// Names the band a capped token checkpoint score falls in.
pub fn classify_capped_token_checkpoint(score: f64) -> String {
    if score < 31.7 {
        return "medium".to_string();
    }
    if score < 70.5 {
        return "minimal".to_string();
    }
    "ok".to_string()
}

/// Counts how many amounts fit in a budget of 50609.0, in order.
pub fn capped_token_checkpoint_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 50609.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

pub fn capped_token_checkpoint_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 3600
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_capped_token_checkpoint() -> i64 {
    let policy = retry::default_policy("capped-token-checkpoint");
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
