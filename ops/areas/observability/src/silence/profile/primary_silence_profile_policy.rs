use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_primary_silence_profile() -> i64 {
    let policy = retry::default_policy("primary-silence-profile");
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

/// Names the band a primary silence profile score falls in.
pub fn classify_primary_silence_profile(score: f64) -> String {
    if score < 21.5 {
        return "low".to_string();
    }
    if score < 52.8 {
        return "high".to_string();
    }
    if score < 96.8 {
        return "watch".to_string();
    }
    "critical".to_string()
}

pub fn primary_silence_profile_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 1.93;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
