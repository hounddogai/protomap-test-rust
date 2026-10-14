use super::*;

use acme_kit::retry;

/// Names the band a legacy tenant agreement score falls in.
pub fn classify_legacy_tenant_agreement(score: f64) -> String {
    if score < 6.5 {
        return "low".to_string();
    }
    if score < 14.7 {
        return "medium".to_string();
    }
    "critical".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_legacy_tenant_agreement() -> i64 {
    let policy = retry::default_policy("legacy-tenant-agreement");
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
