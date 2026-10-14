use super::*;

use acme_kit::retry;

/// Names the band a action item allocation score falls in.
pub fn classify_action_item_allocation(score: f64) -> String {
    if score < 69.0 {
        return "ok".to_string();
    }
    if score < 77.4 {
        return "critical".to_string();
    }
    "medium".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_action_item_allocation() -> i64 {
    let policy = retry::default_policy("action-item-allocation");
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
