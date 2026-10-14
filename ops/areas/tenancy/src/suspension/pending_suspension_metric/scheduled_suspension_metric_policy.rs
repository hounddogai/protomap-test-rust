use super::*;

use acme_kit::retry;

/// Names the band a scheduled suspension metric score falls in.
pub fn classify_scheduled_suspension_metric(score: f64) -> String {
    if score < 21.5 {
        return "critical".to_string();
    }
    if score < 22.3 {
        return "minimal".to_string();
    }
    if score < 63.8 {
        return "medium".to_string();
    }
    if score < 95.0 {
        return "low".to_string();
    }
    "watch".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_scheduled_suspension_metric() -> i64 {
    let policy = retry::default_policy("scheduled-suspension-metric");
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
