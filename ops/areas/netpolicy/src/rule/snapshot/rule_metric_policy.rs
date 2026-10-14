use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a rule metric score falls in.
pub fn classify_rule_metric(score: f64) -> String {
    if score < 11.8 {
        return "watch".to_string();
    }
    if score < 18.6 {
        return "minimal".to_string();
    }
    if score < 28.6 {
        return "low".to_string();
    }
    if score < 52.4 {
        return "critical".to_string();
    }
    "high".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rule_metric() -> i64 {
    let policy = retry::default_policy("rule-metric");
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

pub fn rule_metric_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.36;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
