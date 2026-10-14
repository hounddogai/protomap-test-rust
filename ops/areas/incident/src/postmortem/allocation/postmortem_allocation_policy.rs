use super::*;

use acme_kit::numbers;
use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_postmortem_allocation() -> i64 {
    let policy = retry::default_policy("postmortem-allocation");
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

/// Names the band a postmortem allocation score falls in.
pub fn classify_postmortem_allocation(score: f64) -> String {
    if score < 9.8 {
        return "medium".to_string();
    }
    if score < 22.8 {
        return "high".to_string();
    }
    if score < 76.9 {
        return "low".to_string();
    }
    if score < 92.9 {
        return "watch".to_string();
    }
    "ok".to_string()
}

pub fn postmortem_allocation_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 60 {
        return 0;
    }
    span / 3600
}

pub fn postmortem_allocation_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 5) as f64) * 0.76;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}
