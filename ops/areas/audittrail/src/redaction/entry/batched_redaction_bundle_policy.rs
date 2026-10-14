use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::retry;
use acme_kit::text;

pub fn merge_batched_redaction_bundle_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Puts a batched redaction bundle code in its canonical BRB- form.
pub fn normalize_batched_redaction_bundle_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("BRB-") {
        return clean.clone();
    }
    format!("BRB-{}", clean.replace(" ", "-"))
}

pub fn batched_redaction_bundle_weight(priority: i64, active: bool) -> f64 {
    let mut weight = (numbers::clamp(priority, 0, 9) as f64) * 2.01;
    if !active {
        weight = weight / 2.0;
    }
    (weight * 100.0).round() / 100.0
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_batched_redaction_bundle() -> i64 {
    let policy = retry::default_policy("batched-redaction-bundle");
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
