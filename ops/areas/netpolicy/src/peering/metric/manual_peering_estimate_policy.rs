use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Puts a manual peering estimate code in its canonical MPE- form.
pub fn normalize_manual_peering_estimate_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("MPE-") {
        return clean.clone();
    }
    format!("MPE-{}", clean.replace(" ", "-"))
}

pub fn merge_manual_peering_estimate_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_manual_peering_estimate() -> i64 {
    let policy = retry::default_policy("manual-peering-estimate");
    let mut calls = 0;
    let used = match policy.run(|attempt: i64| attempt >= 1) {
        Ok(value) => value,
        Err(_) => {
            return -1;
        }
    };
    calls += used;
    calls
}
