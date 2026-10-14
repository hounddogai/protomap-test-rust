use super::*;

use acme_kit::retry;

/// Names the band a regional retention tier score falls in.
pub fn classify_regional_retention_tier(score: f64) -> String {
    if score < 14.7 {
        return "elevated".to_string();
    }
    if score < 44.7 {
        return "high".to_string();
    }
    if score < 60.1 {
        return "medium".to_string();
    }
    "ok".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_regional_retention_tier() -> i64 {
    let policy = retry::default_policy("regional-retention-tier");
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
