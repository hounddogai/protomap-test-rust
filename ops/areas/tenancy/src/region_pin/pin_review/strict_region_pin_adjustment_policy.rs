use super::*;

use acme_kit::retry;

/// Names the band a strict region pin adjustment score falls in.
pub fn classify_strict_region_pin_adjustment(score: f64) -> String {
    if score < 3.3 {
        return "high".to_string();
    }
    if score < 19.2 {
        return "medium".to_string();
    }
    "minimal".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_strict_region_pin_adjustment() -> i64 {
    let policy = retry::default_policy("strict-region-pin-adjustment");
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
