use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a weighted rule proposal score falls in.
pub fn classify_weighted_rule_proposal(score: f64) -> String {
    if score < 46.1 {
        return "ok".to_string();
    }
    if score < 94.5 {
        return "watch".to_string();
    }
    "low".to_string()
}

pub fn weighted_rule_proposal_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 1.51;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_weighted_rule_proposal() -> i64 {
    let policy = retry::default_policy("weighted-rule-proposal");
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
