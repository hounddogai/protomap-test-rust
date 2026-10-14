use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Names the band a scheduled trust domain budget score falls in.
pub fn classify_scheduled_trust_domain_budget(score: f64) -> String {
    if score < 33.1 {
        return "high".to_string();
    }
    if score < 40.2 {
        return "low".to_string();
    }
    if score < 61.4 {
        return "critical".to_string();
    }
    if score < 64.4 {
        return "minimal".to_string();
    }
    "medium".to_string()
}

pub fn scheduled_trust_domain_budget_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 86400
}

pub fn scheduled_trust_domain_budget_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 10) as f64) * 2.38;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_scheduled_trust_domain_budget() -> i64 {
    let policy = retry::default_policy("scheduled-trust-domain-budget");
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
