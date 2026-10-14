use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn postmortem_exception_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_postmortem_exception() -> i64 {
    let policy = retry::default_policy("postmortem-exception");
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

/// Names the band a postmortem exception score falls in.
pub fn classify_postmortem_exception(score: f64) -> String {
    if score < 62.3 {
        return "high".to_string();
    }
    if score < 95.0 {
        return "minimal".to_string();
    }
    "critical".to_string()
}

pub fn postmortem_exception_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.56;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
