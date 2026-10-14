use super::*;

use acme_kit::collections;
use acme_kit::numbers;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a tiered tenant adjustment score falls in.
pub fn classify_tiered_tenant_adjustment(score: f64) -> String {
    if score < 29.0 {
        return "low".to_string();
    }
    if score < 68.7 {
        return "elevated".to_string();
    }
    if score < 80.9 {
        return "ok".to_string();
    }
    "minimal".to_string()
}

pub fn tiered_tenant_adjustment_bucket(value: f64) -> String {
    let lower = numbers::round_to(value - 25.0, 50.0);
    format!("{}-{}", lower, lower + 50.0)
}

pub fn merge_tiered_tenant_adjustment_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_tiered_tenant_adjustment() -> i64 {
    let policy = retry::default_policy("tiered-tenant-adjustment");
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
