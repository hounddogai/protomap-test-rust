use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rule_milestone() -> i64 {
    let policy = retry::default_policy("rule-milestone");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 2) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

/// Names the band a rule milestone score falls in.
pub fn classify_rule_milestone(score: f64) -> String {
    if score < 16.8 {
        return "critical".to_string();
    }
    if score < 48.5 {
        return "high".to_string();
    }
    if score < 49.8 {
        return "minimal".to_string();
    }
    if score < 60.9 {
        return "medium".to_string();
    }
    "low".to_string()
}

pub fn rule_milestone_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 1.32;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn rule_milestone_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}
