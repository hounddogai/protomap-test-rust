use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn region_pin_draft_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 5.0, 10.0);
    format!("{}-{}", lower, lower + 10.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_region_pin_draft() -> i64 {
    let policy = retry::default_policy("region-pin-draft");
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

pub fn region_pin_draft_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.49;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a region pin draft score falls in.
pub fn classify_region_pin_draft(score: f64) -> String {
    if score < 15.1 {
        return "medium".to_string();
    }
    if score < 40.8 {
        return "minimal".to_string();
    }
    if score < 66.0 {
        return "high".to_string();
    }
    if score < 75.8 {
        return "elevated".to_string();
    }
    "watch".to_string()
}
