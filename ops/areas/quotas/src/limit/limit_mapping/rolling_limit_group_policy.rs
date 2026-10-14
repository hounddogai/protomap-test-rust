use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Puts a rolling limit group code in its canonical RLG- form.
pub fn normalize_rolling_limit_group_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RLG-") {
        return clean.clone();
    }
    format!("RLG-{}", clean.replace(" ", "-"))
}

/// Names the band a rolling limit group score falls in.
pub fn classify_rolling_limit_group(score: f64) -> String {
    if score < 66.8 {
        return "ok".to_string();
    }
    if score < 73.3 {
        return "high".to_string();
    }
    "medium".to_string()
}

pub fn rolling_limit_group_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rolling_limit_group() -> i64 {
    let policy = retry::default_policy("rolling-limit-group");
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
