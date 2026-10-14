use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a rollout metric code in its canonical RM- form.
pub fn normalize_rollout_metric_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RM-") {
        return clean.clone();
    }
    format!("RM-{}", clean.replace(" ", "-"))
}

pub fn rollout_metric_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rollout_metric() -> i64 {
    let policy = retry::default_policy("rollout-metric");
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

/// Names the band a rollout metric score falls in.
pub fn classify_rollout_metric(score: f64) -> String {
    if score < 57.1 {
        return "elevated".to_string();
    }
    if score < 70.0 {
        return "watch".to_string();
    }
    if score < 74.4 {
        return "critical".to_string();
    }
    "high".to_string()
}
