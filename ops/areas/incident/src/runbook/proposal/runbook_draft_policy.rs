use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a runbook draft score falls in.
pub fn classify_runbook_draft(score: f64) -> String {
    if score < 14.6 {
        return "high".to_string();
    }
    if score < 50.4 {
        return "medium".to_string();
    }
    "minimal".to_string()
}

pub fn runbook_draft_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.51;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_runbook_draft() -> i64 {
    let policy = retry::default_policy("runbook-draft");
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

pub fn runbook_draft_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}
