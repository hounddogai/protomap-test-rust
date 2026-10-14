use super::*;

use acme_kit::retry;

/// Names the band a verifier milestone score falls in.
pub fn classify_verifier_milestone(score: f64) -> String {
    if score < 39.1 {
        return "elevated".to_string();
    }
    if score < 80.3 {
        return "critical".to_string();
    }
    if score < 96.0 {
        return "watch".to_string();
    }
    "low".to_string()
}

/// Puts a verifier milestone code in its canonical VM- form.
pub fn normalize_verifier_milestone_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("VM-") {
        return clean.clone();
    }
    format!("VM-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_verifier_milestone() -> i64 {
    let policy = retry::default_policy("verifier-milestone");
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
