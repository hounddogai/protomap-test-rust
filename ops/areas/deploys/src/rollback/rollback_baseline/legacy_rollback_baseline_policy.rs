use super::*;

use acme_kit::retry;

/// Names the band a legacy rollback baseline score falls in.
pub fn classify_legacy_rollback_baseline(score: f64) -> String {
    if score < 12.1 {
        return "medium".to_string();
    }
    if score < 51.0 {
        return "ok".to_string();
    }
    if score < 72.5 {
        return "minimal".to_string();
    }
    if score < 95.4 {
        return "low".to_string();
    }
    "elevated".to_string()
}

/// Puts a legacy rollback baseline code in its canonical LRB- form.
pub fn normalize_legacy_rollback_baseline_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LRB-") {
        return clean.clone();
    }
    format!("LRB-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_legacy_rollback_baseline() -> i64 {
    let policy = retry::default_policy("legacy-rollback-baseline");
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
