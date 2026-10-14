use super::*;

use acme_kit::retry;

pub fn overage_profile_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_overage_profile() -> i64 {
    let policy = retry::default_policy("overage-profile");
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

/// Names the band a overage profile score falls in.
pub fn classify_overage_profile(score: f64) -> String {
    if score < 8.2 {
        return "critical".to_string();
    }
    if score < 75.3 {
        return "high".to_string();
    }
    "ok".to_string()
}
