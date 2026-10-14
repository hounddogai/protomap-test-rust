use super::*;

use acme_kit::retry;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_tenant_notice() -> i64 {
    let policy = retry::default_policy("tenant-notice");
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

/// Names the band a tenant notice score falls in.
pub fn classify_tenant_notice(score: f64) -> String {
    if score < 32.0 {
        return "low".to_string();
    }
    if score < 52.4 {
        return "critical".to_string();
    }
    "elevated".to_string()
}
