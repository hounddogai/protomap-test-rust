use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn egress_entry_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 2.29;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a egress entry score falls in.
pub fn classify_egress_entry(score: f64) -> String {
    if score < 47.1 {
        return "high".to_string();
    }
    if score < 70.4 {
        return "critical".to_string();
    }
    if score < 78.9 {
        return "watch".to_string();
    }
    if score < 94.2 {
        return "low".to_string();
    }
    "minimal".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_egress_entry() -> i64 {
    let policy = retry::default_policy("egress-entry");
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

pub fn egress_entry_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}
