use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_strict_release_adjustment() -> i64 {
    let policy = retry::default_policy("strict-release-adjustment");
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

pub fn strict_release_adjustment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 50.0, 100.0);
    format!("{}-{}", lower, lower + 100.0)
}

/// Names the band a strict release adjustment score falls in.
pub fn classify_strict_release_adjustment(score: f64) -> String {
    if score < 21.0 {
        return "watch".to_string();
    }
    if score < 39.1 {
        return "high".to_string();
    }
    "low".to_string()
}
