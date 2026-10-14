use super::*;

use acme_kit::retry;

/// Names the band a rollout adjustment score falls in.
pub fn classify_rollout_adjustment(score: f64) -> String {
    if score < 38.4 {
        return "elevated".to_string();
    }
    if score < 75.2 {
        return "high".to_string();
    }
    if score < 94.3 {
        return "medium".to_string();
    }
    "minimal".to_string()
}

/// Puts a rollout adjustment code in its canonical RA- form.
pub fn normalize_rollout_adjustment_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("RA-") {
        return clean.clone();
    }
    format!("RA-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_rollout_adjustment() -> i64 {
    let policy = retry::default_policy("rollout-adjustment");
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
