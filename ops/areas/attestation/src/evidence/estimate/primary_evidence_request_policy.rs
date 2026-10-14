use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_primary_evidence_request() -> i64 {
    let policy = retry::default_policy("primary-evidence-request");
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

/// Names the band a primary evidence request score falls in.
pub fn classify_primary_evidence_request(score: f64) -> String {
    if score < 26.1 {
        return "medium".to_string();
    }
    if score < 31.3 {
        return "low".to_string();
    }
    if score < 34.6 {
        return "critical".to_string();
    }
    if score < 52.9 {
        return "high".to_string();
    }
    "minimal".to_string()
}
