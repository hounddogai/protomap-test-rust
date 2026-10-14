use super::*;

use acme_kit::retry;

/// Puts a regional failover checkpoint code in its canonical RFC- form.
pub fn normalize_regional_failover_checkpoint_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RFC-") {
        return clean.clone();
    }
    format!("RFC-{}", clean.replace(" ", "-"))
}

/// Names the band a regional failover checkpoint score falls in.
pub fn classify_regional_failover_checkpoint(score: f64) -> String {
    if score < 25.4 {
        return "watch".to_string();
    }
    if score < 45.9 {
        return "minimal".to_string();
    }
    if score < 46.9 {
        return "high".to_string();
    }
    "low".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_regional_failover_checkpoint() -> i64 {
    let policy = retry::default_policy("regional-failover-checkpoint");
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
