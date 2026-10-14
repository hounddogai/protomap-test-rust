use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_manual_allowlist_forecast() -> i64 {
    let policy = retry::default_policy("manual-allowlist-forecast");
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

/// Counts how many amounts fit in a budget of 39572.0, in order.
pub fn manual_allowlist_forecast_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 39572.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a manual allowlist forecast score falls in.
pub fn classify_manual_allowlist_forecast(score: f64) -> String {
    if score < 13.3 {
        return "watch".to_string();
    }
    if score < 21.8 {
        return "medium".to_string();
    }
    if score < 59.4 {
        return "ok".to_string();
    }
    "high".to_string()
}
