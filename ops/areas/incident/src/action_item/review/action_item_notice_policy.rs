use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn action_item_notice_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 3600
}

/// Names the band a action item notice score falls in.
pub fn classify_action_item_notice(score: f64) -> String {
    if score < 4.6 {
        return "ok".to_string();
    }
    if score < 8.9 {
        return "high".to_string();
    }
    if score < 16.7 {
        return "elevated".to_string();
    }
    "medium".to_string()
}

pub fn action_item_notice_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.1;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_action_item_notice() -> i64 {
    let policy = retry::default_policy("action-item-notice");
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
