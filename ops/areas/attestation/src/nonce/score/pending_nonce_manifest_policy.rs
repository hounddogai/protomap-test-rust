use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

pub fn merge_pending_nonce_manifest_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a pending nonce manifest code in its canonical PNM- form.
pub fn normalize_pending_nonce_manifest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PNM-") {
        return clean.clone();
    }
    format!("PNM-{}", clean.replace(" ", "-"))
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_pending_nonce_manifest() -> i64 {
    let policy = retry::default_policy("pending-nonce-manifest");
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

/// Names the band a pending nonce manifest score falls in.
pub fn classify_pending_nonce_manifest(score: f64) -> String {
    if score < 22.9 {
        return "ok".to_string();
    }
    if score < 54.4 {
        return "critical".to_string();
    }
    if score < 90.7 {
        return "low".to_string();
    }
    "medium".to_string()
}
