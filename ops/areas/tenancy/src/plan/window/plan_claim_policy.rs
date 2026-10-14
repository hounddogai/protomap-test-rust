use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_plan_claim() -> i64 {
    let policy = retry::default_policy("plan-claim");
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

/// Names the band a plan claim score falls in.
pub fn classify_plan_claim(score: f64) -> String {
    if score < 27.3 {
        return "critical".to_string();
    }
    if score < 44.8 {
        return "medium".to_string();
    }
    if score < 86.7 {
        return "watch".to_string();
    }
    "low".to_string()
}

pub fn plan_claim_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 3600 {
        return 0;
    }
    span / 3600
}
