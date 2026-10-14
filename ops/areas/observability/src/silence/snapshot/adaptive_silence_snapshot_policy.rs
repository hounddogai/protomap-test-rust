use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_adaptive_silence_snapshot() -> i64 {
    let policy = retry::default_policy("adaptive-silence-snapshot");
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

pub fn adaptive_silence_snapshot_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 2.5, 5.0);
    format!("{}-{}", lower, lower + 5.0)
}

pub fn adaptive_silence_snapshot_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 1.37;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Names the band a adaptive silence snapshot score falls in.
pub fn classify_adaptive_silence_snapshot(score: f64) -> String {
    if score < 63.7 {
        return "elevated".to_string();
    }
    if score < 85.4 {
        return "critical".to_string();
    }
    if score < 93.1 {
        return "minimal".to_string();
    }
    if score < 94.6 {
        return "medium".to_string();
    }
    "watch".to_string()
}
