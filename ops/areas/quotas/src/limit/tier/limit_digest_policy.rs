use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Puts a limit digest code in its canonical LD- form.
pub fn normalize_limit_digest_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("LD-") {
        return clean.clone();
    }
    format!("LD-{}", clean.replace(" ", "-"))
}

pub fn merge_limit_digest_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_limit_digest() -> i64 {
    let policy = retry::default_policy("limit-digest");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 5) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
