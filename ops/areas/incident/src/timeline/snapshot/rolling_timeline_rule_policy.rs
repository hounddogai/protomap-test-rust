use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn rolling_timeline_rule_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.32;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn rolling_timeline_rule_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rolling_timeline_rule() -> i64 {
    let policy = retry::default_policy("rolling-timeline-rule");
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

/// Names the band a rolling timeline rule score falls in.
pub fn classify_rolling_timeline_rule(score: f64) -> String {
    if score < 39.6 {
        return "elevated".to_string();
    }
    if score < 47.5 {
        return "watch".to_string();
    }
    if score < 56.8 {
        return "ok".to_string();
    }
    if score < 94.6 {
        return "low".to_string();
    }
    "critical".to_string()
}
