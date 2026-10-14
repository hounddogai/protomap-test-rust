use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn global_grant_metric_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.26;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn global_grant_metric_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_global_grant_metric() -> i64 {
    let policy = retry::default_policy("global-grant-metric");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 2) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Names the band a global grant metric score falls in.
pub fn classify_global_grant_metric(score: f64) -> String {
    if score < 17.6 {
        return "low".to_string();
    }
    if score < 46.5 {
        return "watch".to_string();
    }
    if score < 72.4 {
        return "high".to_string();
    }
    "minimal".to_string()
}
