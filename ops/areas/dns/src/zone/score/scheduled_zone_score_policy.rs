use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_scheduled_zone_score() -> i64 {
    let policy = retry::default_policy("scheduled-zone-score");
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

pub fn scheduled_zone_score_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.29;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a scheduled zone score score falls in.
pub fn classify_scheduled_zone_score(score: f64) -> String {
    if score < 61.8 {
        return "minimal".to_string();
    }
    if score < 64.0 {
        return "watch".to_string();
    }
    "elevated".to_string()
}
