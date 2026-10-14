use super::*;

use acme_kit::collections;
use acme_kit::retry;
use acme_kit::text;

/// Names the band a pending migration group score falls in.
pub fn classify_pending_migration_group(score: f64) -> String {
    if score < 42.2 {
        return "critical".to_string();
    }
    if score < 47.9 {
        return "watch".to_string();
    }
    "high".to_string()
}

/// Puts a pending migration group code in its canonical PMG- form.
pub fn normalize_pending_migration_group_code(code: &str) -> String {
    let clean = code.trim().to_uppercase();
    if clean == "" {
        return "".to_string();
    }
    if clean.starts_with("PMG-") {
        return clean.clone();
    }
    format!("PMG-{}", clean.replace(" ", "-"))
}

pub fn merge_pending_migration_group_labels(left: &[String], right: &[String]) -> Vec<String> {
    let mut merged = left.to_vec();
    for tag in right.iter() {
        merged.push(text::normalize(tag));
    }
    collections::unique_strings(&merged)
}

/// Runs the default retry policy against a probe that succeeds late.
pub fn probe_pending_migration_group() -> i64 {
    let policy = retry::default_policy("pending-migration-group");
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
