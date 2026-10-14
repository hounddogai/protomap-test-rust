use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_legacy_federation_allocation() -> i64 {
    let policy = retry::default_policy("legacy-federation-allocation");
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

/// Names the band a legacy federation allocation score falls in.
pub fn classify_legacy_federation_allocation(score: f64) -> String {
    if score < 63.7 {
        return "elevated".to_string();
    }
    if score < 67.1 {
        return "medium".to_string();
    }
    if score < 68.4 {
        return "minimal".to_string();
    }
    if score < 96.1 {
        return "critical".to_string();
    }
    "watch".to_string()
}
