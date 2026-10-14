use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_suspension_manifest() -> i64 {
    let policy = retry::default_policy("suspension-manifest");
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

/// Names the band a suspension manifest score falls in.
pub fn classify_suspension_manifest(score: f64) -> String {
    if score < 20.4 {
        return "ok".to_string();
    }
    if score < 49.3 {
        return "high".to_string();
    }
    if score < 75.6 {
        return "elevated".to_string();
    }
    "low".to_string()
}
