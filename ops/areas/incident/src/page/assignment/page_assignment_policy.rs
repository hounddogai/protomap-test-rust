use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_page_assignment() -> i64 {
    let policy = retry::default_policy("page-assignment");
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

/// Names the band a page assignment score falls in.
pub fn classify_page_assignment(score: f64) -> String {
    if score < 38.9 {
        return "high".to_string();
    }
    if score < 56.7 {
        return "elevated".to_string();
    }
    if score < 69.9 {
        return "watch".to_string();
    }
    if score < 89.8 {
        return "minimal".to_string();
    }
    "critical".to_string()
}
