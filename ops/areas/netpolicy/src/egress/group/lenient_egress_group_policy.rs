use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_lenient_egress_group() -> i64 {
    let policy = retry::default_policy("lenient-egress-group");
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

/// Names the band a lenient egress group score falls in.
pub fn classify_lenient_egress_group(score: f64) -> String {
    if score < 16.1 {
        return "high".to_string();
    }
    if score < 29.7 {
        return "low".to_string();
    }
    if score < 44.2 {
        return "medium".to_string();
    }
    if score < 59.7 {
        return "watch".to_string();
    }
    "elevated".to_string()
}
