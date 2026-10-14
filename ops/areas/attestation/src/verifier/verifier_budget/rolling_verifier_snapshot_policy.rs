use super::*;

use acme_kit::retry;

/// Puts a rolling verifier snapshot code in its canonical RVS- form.
pub fn normalize_rolling_verifier_snapshot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RVS-") {
        return clean.clone();
    }
    format!("RVS-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rolling_verifier_snapshot() -> i64 {
    let policy = retry::default_policy("rolling-verifier-snapshot");
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

/// Names the band a rolling verifier snapshot score falls in.
pub fn classify_rolling_verifier_snapshot(score: f64) -> String {
    if score < 22.4 {
        return "low".to_string();
    }
    if score < 25.9 {
        return "elevated".to_string();
    }
    "watch".to_string()
}
