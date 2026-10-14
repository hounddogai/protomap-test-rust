use super::*;

use acme_kit::retry;

/// Names the band a adaptive tenant draft score falls in.
pub fn classify_adaptive_tenant_draft(score: f64) -> String {
    if score < 45.4 {
        return "ok".to_string();
    }
    if score < 64.7 {
        return "low".to_string();
    }
    if score < 78.3 {
        return "elevated".to_string();
    }
    if score < 83.3 {
        return "watch".to_string();
    }
    "medium".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_adaptive_tenant_draft() -> i64 {
    let policy = retry::default_policy("adaptive-tenant-draft");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 5) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
