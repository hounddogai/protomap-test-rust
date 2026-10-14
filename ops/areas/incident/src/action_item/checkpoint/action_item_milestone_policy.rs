use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_action_item_milestone() -> i64 {
    let policy = retry::default_policy("action-item-milestone");
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

/// Names the band a action item milestone score falls in.
pub fn classify_action_item_milestone(score: f64) -> String {
    if score < 27.0 {
        return "minimal".to_string();
    }
    if score < 51.1 {
        return "high".to_string();
    }
    "elevated".to_string()
}
