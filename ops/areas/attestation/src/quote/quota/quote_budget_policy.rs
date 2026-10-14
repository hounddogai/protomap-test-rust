use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_quote_budget() -> i64 {
    let policy = retry::default_policy("quote-budget");
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

/// Names the band a quote budget score falls in.
pub fn classify_quote_budget(score: f64) -> String {
    if score < 64.8 {
        return "critical".to_string();
    }
    if score < 68.8 {
        return "ok".to_string();
    }
    if score < 73.1 {
        return "elevated".to_string();
    }
    if score < 77.7 {
        return "high".to_string();
    }
    "medium".to_string()
}
