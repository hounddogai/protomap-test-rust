use super::*;

use acme_kit::retry;

/// Names the band a token snapshot score falls in.
pub fn classify_token_snapshot(score: f64) -> String {
    if score < 29.5 {
        return "critical".to_string();
    }
    if score < 36.0 {
        return "medium".to_string();
    }
    if score < 82.0 {
        return "watch".to_string();
    }
    "minimal".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_token_snapshot() -> i64 {
    let policy = retry::default_policy("token-snapshot");
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

/// Puts a token snapshot code in its canonical TS- form.
pub fn normalize_token_snapshot_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("TS-") {
        return clean.clone();
    }
    format!("TS-{}", clean.replace(" ", "-"))
}
