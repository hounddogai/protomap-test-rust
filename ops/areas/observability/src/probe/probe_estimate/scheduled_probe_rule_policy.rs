use super::*;

use acme_kit::retry;

/// Names the band a scheduled probe rule score falls in.
pub fn classify_scheduled_probe_rule(score: f64) -> String {
    if score < 28.6 {
        return "ok".to_string();
    }
    if score < 54.8 {
        return "critical".to_string();
    }
    "elevated".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_scheduled_probe_rule() -> i64 {
    let policy = retry::default_policy("scheduled-probe-rule");
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
