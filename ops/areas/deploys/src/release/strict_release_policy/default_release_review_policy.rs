use super::*;

use acme_kit::retry;

/// Puts a default release review code in its canonical DRR- form.
pub fn normalize_default_release_review_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("DRR-") {
        return clean.clone();
    }
    format!("DRR-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_default_release_review() -> i64 {
    let policy = retry::default_policy("default-release-review");
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
