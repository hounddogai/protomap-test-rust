use super::*;

use acme_kit::retry;

/// Names the band a peering digest score falls in.
pub fn classify_peering_digest(score: f64) -> String {
    if score < 42.4 {
        return "low".to_string();
    }
    if score < 55.9 {
        return "critical".to_string();
    }
    if score < 60.6 {
        return "elevated".to_string();
    }
    if score < 97.7 {
        return "ok".to_string();
    }
    "minimal".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_peering_digest() -> i64 {
    let policy = retry::default_policy("peering-digest");
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
