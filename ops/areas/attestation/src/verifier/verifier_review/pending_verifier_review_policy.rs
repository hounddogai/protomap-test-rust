use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Puts a pending verifier review code in its canonical PVR- form.
pub fn normalize_pending_verifier_review_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PVR-") {
        return clean.clone();
    }
    format!("PVR-{}", clean.replace(" ", "-"))
}

pub fn merge_pending_verifier_review_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_pending_verifier_review() -> i64 {
    let policy = retry::default_policy("pending-verifier-review");
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
