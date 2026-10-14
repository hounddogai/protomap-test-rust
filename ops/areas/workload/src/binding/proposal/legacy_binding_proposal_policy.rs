use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Puts a legacy binding proposal code in its canonical LBP- form.
pub fn normalize_legacy_binding_proposal_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LBP-") {
        return clean.clone();
    }
    format!("LBP-{}", clean.replace(" ", "-"))
}

pub fn merge_legacy_binding_proposal_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Names the band a legacy binding proposal score falls in.
pub fn classify_legacy_binding_proposal(score: f64) -> String {
    if score < 16.6 {
        return "watch".to_string();
    }
    if score < 64.7 {
        return "high".to_string();
    }
    if score < 97.5 {
        return "elevated".to_string();
    }
    "minimal".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_legacy_binding_proposal() -> i64 {
    let policy = retry::default_policy("legacy-binding-proposal");
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
