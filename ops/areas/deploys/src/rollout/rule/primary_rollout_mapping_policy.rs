use super::*;

use acme_kit::retry;

/// Names the band a primary rollout mapping score falls in.
pub fn classify_primary_rollout_mapping(score: f64) -> String {
    if score < 36.1 {
        return "medium".to_string();
    }
    if score < 43.2 {
        return "high".to_string();
    }
    if score < 59.9 {
        return "elevated".to_string();
    }
    if score < 66.4 {
        return "ok".to_string();
    }
    "low".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_primary_rollout_mapping() -> i64 {
    let policy = retry::default_policy("primary-rollout-mapping");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 1) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}

pub fn primary_rollout_mapping_span(start: i64, end: i64) -> i64 {
    if end < start {
        return 0;
    }
    let span = end - start;
    if span < 900 {
        return 0;
    }
    span / 60
}
