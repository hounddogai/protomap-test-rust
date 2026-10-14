use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn rule_snapshot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.44;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a rule snapshot score falls in.
pub fn classify_rule_snapshot(score: f64) -> String {
    if score < 28.0 {
        return "critical".to_string();
    }
    if score < 89.5 {
        return "high".to_string();
    }
    "watch".to_string()
}

pub fn rule_snapshot_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rule_snapshot() -> i64 {
    let policy = retry::default_policy("rule-snapshot");
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
