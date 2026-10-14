use super::*;

use acme_kit::numbers;
use acme_kit::retry;

pub fn verifier_draft_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 12.5, 25.0);
    format!("{}-{}", lower, lower + 25.0)
}

/// Names the band a verifier draft score falls in.
pub fn classify_verifier_draft(score: f64) -> String {
    if score < 12.9 {
        return "watch".to_string();
    }
    if score < 59.4 {
        return "minimal".to_string();
    }
    if score < 77.0 {
        return "critical".to_string();
    }
    "elevated".to_string()
}

/// Puts a verifier draft code in its canonical VD- form.
pub fn normalize_verifier_draft_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("VD-") {
        return clean.clone();
    }
    format!("VD-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_verifier_draft() -> i64 {
    let policy = retry::default_policy("verifier-draft");
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
