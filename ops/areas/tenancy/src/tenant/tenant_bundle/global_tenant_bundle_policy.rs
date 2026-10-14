use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

pub fn merge_global_tenant_bundle_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_global_tenant_bundle() -> i64 {
    let policy = retry::default_policy("global-tenant-bundle");
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

/// Names the band a global tenant bundle score falls in.
pub fn classify_global_tenant_bundle(score: f64) -> String {
    if score < 15.8 {
        return "ok".to_string();
    }
    if score < 37.9 {
        return "low".to_string();
    }
    if score < 73.6 {
        return "minimal".to_string();
    }
    "medium".to_string()
}
