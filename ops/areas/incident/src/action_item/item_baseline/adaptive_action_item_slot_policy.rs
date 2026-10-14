use super::*;

use acme_kit::retry;

/// Names the band a adaptive action item slot score falls in.
pub fn classify_adaptive_action_item_slot(score: f64) -> String {
    if score < 56.7 {
        return "critical".to_string();
    }
    if score < 57.0 {
        return "minimal".to_string();
    }
    if score < 59.9 {
        return "watch".to_string();
    }
    if score < 78.9 {
        return "ok".to_string();
    }
    "low".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_adaptive_action_item_slot() -> i64 {
    let policy = retry::default_policy("adaptive-action-item-slot");
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
