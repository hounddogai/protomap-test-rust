use super::*;

use acme_kit::retry;

/// Puts a legacy allowlist checkpoint code in its canonical LAC- form.
pub fn normalize_legacy_allowlist_checkpoint_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LAC-") {
        return clean.clone();
    }
    format!("LAC-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_legacy_allowlist_checkpoint() -> i64 {
    let policy = retry::default_policy("legacy-allowlist-checkpoint");
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

/// Names the band a legacy allowlist checkpoint score falls in.
pub fn classify_legacy_allowlist_checkpoint(score: f64) -> String {
    if score < 13.6 {
        return "watch".to_string();
    }
    if score < 47.4 {
        return "medium".to_string();
    }
    "critical".to_string()
}
