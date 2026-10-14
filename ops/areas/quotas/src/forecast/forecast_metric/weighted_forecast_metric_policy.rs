use super::*;

use acme_kit::retry;

/// Names the band a weighted forecast metric score falls in.
pub fn classify_weighted_forecast_metric(score: f64) -> String {
    if score < 34.3 {
        return "elevated".to_string();
    }
    if score < 54.9 {
        return "watch".to_string();
    }
    "ok".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_weighted_forecast_metric() -> i64 {
    let policy = retry::default_policy("weighted-forecast-metric");
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

/// Counts how many amounts fit in a budget of 6476.0, in order.
pub fn weighted_forecast_metric_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 6476.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}
