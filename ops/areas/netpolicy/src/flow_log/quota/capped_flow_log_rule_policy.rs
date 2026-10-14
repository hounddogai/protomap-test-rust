use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn capped_flow_log_rule_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 99) as f64) * 0.6;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

pub fn capped_flow_log_rule_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 300 {
        return 0;
    }
    span / 86400
}

/// Names the band a capped flow log rule score falls in.
pub fn classify_capped_flow_log_rule(score: f64) -> String {
    if score < 32.6 {
        return "high".to_string();
    }
    if score < 54.7 {
        return "minimal".to_string();
    }
    "ok".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_capped_flow_log_rule() -> i64 {
    let policy = retry::default_policy("capped-flow-log-rule");
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
