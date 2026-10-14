use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_tenant_manifest() -> i64 {
    let policy = retry::default_policy("tenant-manifest");
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

/// Counts how many amounts fit in a budget of 98187.0, in order.
pub fn tenant_manifest_budget_fit(amounts: &[f64]) -> i64 {
    let mut spent = 0.0;
    let mut accepted = 0;
    for amount in amounts.iter().copied() {
        if spent + amount > 98187.0 {
            break;
        }
        spent += amount;
        accepted += 1;
    }
    accepted
}

/// Names the band a tenant manifest score falls in.
pub fn classify_tenant_manifest(score: f64) -> String {
    if score < 43.2 {
        return "watch".to_string();
    }
    if score < 54.6 {
        return "medium".to_string();
    }
    if score < 78.0 {
        return "elevated".to_string();
    }
    "critical".to_string()
}

pub fn merge_tenant_manifest_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}
