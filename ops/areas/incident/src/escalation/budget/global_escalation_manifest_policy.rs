use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a global escalation manifest score falls in.
pub fn classify_global_escalation_manifest(score: f64) -> String {
    if score < 3.1 {
        return "medium".to_string();
    }
    if score < 38.1 {
        return "ok".to_string();
    }
    if score < 67.0 {
        return "watch".to_string();
    }
    if score < 68.6 {
        return "critical".to_string();
    }
    "high".to_string()
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_global_escalation_manifest() -> i64 {
    let policy = retry::default_policy("global-escalation-manifest");
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

pub fn merge_global_escalation_manifest_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
